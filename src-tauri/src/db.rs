use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{
    ClientGroup, DrawerSettings, MumbleLink, MumbleLinksOverlaySettings, MumbleServerGroup, Profile,
    ThumbnailConfig, ThumbnailSetting,
};

pub struct DbService {
    db_path: PathBuf,
}

impl DbService {
    pub fn new() -> Result<Self, String> {
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let db_path = if cwd
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("src-tauri"))
        {
            cwd.join("settings.db")
        } else if cwd.join("src-tauri").is_dir() {
            cwd.join("src-tauri").join("settings.db")
        } else {
            cwd.join("settings.db")
        };

        let service = Self { db_path };
        service.initialize()?;
        Ok(service)
    }

    fn connection(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|e| e.to_string())
    }

    fn initialize(&self) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute_batch(include_str!("../sql/bootstrap.sql"))
            .map_err(|e| e.to_string())?;
        conn.execute_batch(include_str!("../sql/mumble_and_groups.sql"))
            .map_err(|e| e.to_string())?;
        self.bootstrap_defaults(&conn)
    }

    fn bootstrap_defaults(&self, conn: &Connection) -> Result<(), String> {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM Profile", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if count > 0 {
            return Ok(());
        }

        conn.execute(
            "INSERT INTO Profile (Name, IsActive, SwitchHotkey) VALUES (?1, 1, '')",
            ["Default"],
        )
        .map_err(|e| e.to_string())?;
        let profile_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO ThumbnailDefaultConfig
            (ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay)
            VALUES (?1, 400, 300, 100, 100, 0.75, '#0078D4', 3, 1)",
            [profile_id],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO ProcessesToPreview (ProfileId, ProcessName) VALUES (?1, 'exefile')",
            [profile_id],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO ClientGroups (ProfileId, Name, DisplayOrder, CycleForwardHotkey, CycleBackwardHotkey)
             VALUES (?1, 'Default', 0, '', '')",
            [profile_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_profiles(&self) -> Result<Vec<Profile>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT Id, Name, DeletedAt, IsActive, SwitchHotkey
                 FROM Profile
                 ORDER BY Name ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Profile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    deleted_at: row.get(2)?,
                    is_active: row.get::<_, i64>(3)? == 1,
                    switch_hotkey: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn create_profile(&self, name: String) -> Result<Profile, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Profile name cannot be empty".to_string());
        }
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO Profile (Name, IsActive, SwitchHotkey) VALUES (?1, 0, '')",
            [trimmed],
        )
        .map_err(|e| e.to_string())?;
        let profile_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT OR IGNORE INTO ThumbnailDefaultConfig
            (ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay)
            VALUES (?1, 400, 300, 100, 100, 0.75, '#0078D4', 3, 1)",
            [profile_id],
        )
        .map_err(|e| e.to_string())?;
        self.get_profile_by_id(profile_id)
    }

    fn get_profile_by_id(&self, profile_id: i64) -> Result<Profile, String> {
        let conn = self.connection()?;
        conn.query_row(
            "SELECT Id, Name, DeletedAt, IsActive, SwitchHotkey FROM Profile WHERE Id = ?1",
            [profile_id],
            |row| {
                Ok(Profile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    deleted_at: row.get(2)?,
                    is_active: row.get::<_, i64>(3)? == 1,
                    switch_hotkey: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn set_active_profile(&self, profile_id: i64) -> Result<(), String> {
        let conn = self.connection()?;
        let exists: Option<i64> = conn
            .query_row("SELECT Id FROM Profile WHERE Id = ?1 LIMIT 1", [profile_id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        if exists.is_none() {
            return Err("Profile not found".to_string());
        }
        conn.execute("UPDATE Profile SET IsActive = 0", [])
            .map_err(|e| e.to_string())?;
        conn.execute("UPDATE Profile SET IsActive = 1 WHERE Id = ?1", [profile_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_profile_hotkey(&self, profile_id: i64, hotkey: String) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE Profile SET SwitchHotkey = ?1 WHERE Id = ?2",
            params![hotkey, profile_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_profile(&self, profile_id: i64) -> Result<(), String> {
        let conn = self.connection()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM Profile", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if count <= 1 {
            return Err("Cannot delete the last profile".to_string());
        }
        let is_active: Option<i64> = conn
            .query_row(
                "SELECT Id FROM Profile WHERE Id = ?1 AND IsActive = 1 LIMIT 1",
                [profile_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if is_active.is_some() {
            return Err("Cannot delete the active profile".to_string());
        }
        conn.execute("DELETE FROM Profile WHERE Id = ?1", [profile_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn active_profile_id(&self) -> Option<i64> {
        let conn = self.connection().ok()?;
        conn.query_row("SELECT Id FROM Profile WHERE IsActive = 1 LIMIT 1", [], |r| {
            r.get(0)
        })
        .optional()
        .ok()
        .flatten()
    }

    pub fn get_processes_to_preview(&self, profile_id: i64) -> Result<Vec<String>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare("SELECT ProcessName FROM ProcessesToPreview WHERE ProfileId = ?1 ORDER BY ProcessName")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([profile_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn add_process_to_preview(&self, profile_id: i64, process_name: String) -> Result<(), String> {
        let value = process_name.trim().to_lowercase();
        if value.is_empty() {
            return Err("Process name cannot be empty".to_string());
        }
        let conn = self.connection()?;
        conn.execute(
            "INSERT OR IGNORE INTO ProcessesToPreview (ProfileId, ProcessName) VALUES (?1, ?2)",
            params![profile_id, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_process_to_preview(&self, profile_id: i64, process_name: String) -> Result<(), String> {
        let normalized = process_name.trim().to_lowercase();
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM ProcessesToPreview WHERE ProfileId = ?1 AND ProcessName = ?2",
            params![profile_id, normalized.clone()],
        )
        .map_err(|e| e.to_string())?;
        // Heuristic cleanup: if a title contains the removed process name, remove that override.
        conn.execute(
            "DELETE FROM ThumbnailSettings WHERE ProfileId = ?1 AND lower(WindowTitle) LIKE ?2",
            params![profile_id, format!("%{normalized}%")],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_thumbnail_default_config(&self, profile_id: i64) -> Result<ThumbnailConfig, String> {
        let conn = self.connection()?;
        conn.query_row(
            "SELECT Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay
             FROM ThumbnailDefaultConfig WHERE ProfileId = ?1",
            [profile_id],
            |row| {
                Ok(ThumbnailConfig {
                    width: row.get(0)?,
                    height: row.get(1)?,
                    x: row.get(2)?,
                    y: row.get(3)?,
                    opacity: row.get(4)?,
                    focus_border_color: row.get(5)?,
                    focus_border_thickness: row.get(6)?,
                    show_title_overlay: row.get::<_, i64>(7)? == 1,
                })
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn set_thumbnail_default_config(
        &self,
        profile_id: i64,
        config: ThumbnailConfig,
    ) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO ThumbnailDefaultConfig
            (ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(ProfileId) DO UPDATE SET
              Width=excluded.Width,
              Height=excluded.Height,
              X=excluded.X,
              Y=excluded.Y,
              Opacity=excluded.Opacity,
              FocusBorderColor=excluded.FocusBorderColor,
              FocusBorderThickness=excluded.FocusBorderThickness,
              ShowTitleOverlay=excluded.ShowTitleOverlay",
            params![
                profile_id,
                config.width,
                config.height,
                config.x,
                config.y,
                config.opacity,
                config.focus_border_color,
                config.focus_border_thickness,
                if config.show_title_overlay { 1 } else { 0 }
            ],
        )
        .map_err(|e| e.to_string())?;
        // Keep per-window X/Y, but synchronize other values to updated defaults.
        conn.execute(
            "UPDATE ThumbnailSettings
             SET Width = ?2,
                 Height = ?3,
                 Opacity = ?4,
                 FocusBorderColor = ?5,
                 FocusBorderThickness = ?6,
                 ShowTitleOverlay = ?7
             WHERE ProfileId = ?1",
            params![
                profile_id,
                config.width,
                config.height,
                config.opacity,
                config.focus_border_color,
                config.focus_border_thickness,
                if config.show_title_overlay { 1 } else { 0 }
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_thumbnail_settings(&self, profile_id: i64) -> Result<Vec<ThumbnailSetting>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT WindowTitle, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay
                 FROM ThumbnailSettings WHERE ProfileId = ?1 ORDER BY WindowTitle",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([profile_id], |row| {
                Ok(ThumbnailSetting {
                    window_title: row.get(0)?,
                    config: ThumbnailConfig {
                        width: row.get(1)?,
                        height: row.get(2)?,
                        x: row.get(3)?,
                        y: row.get(4)?,
                        opacity: row.get(5)?,
                        focus_border_color: row.get(6)?,
                        focus_border_thickness: row.get(7)?,
                        show_title_overlay: row.get::<_, i64>(8)? == 1,
                    },
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn save_thumbnail_setting(
        &self,
        profile_id: i64,
        window_title: String,
        config: ThumbnailConfig,
    ) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO ThumbnailSettings
             (ProfileId, WindowTitle, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(ProfileId, WindowTitle) DO UPDATE SET
               Width=excluded.Width, Height=excluded.Height, X=excluded.X, Y=excluded.Y,
               Opacity=excluded.Opacity, FocusBorderColor=excluded.FocusBorderColor,
               FocusBorderThickness=excluded.FocusBorderThickness, ShowTitleOverlay=excluded.ShowTitleOverlay",
            params![
                profile_id,
                window_title,
                config.width,
                config.height,
                config.x,
                config.y,
                config.opacity,
                config.focus_border_color,
                config.focus_border_thickness,
                if config.show_title_overlay { 1 } else { 0 }
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_client_groups(&self, profile_id: i64) -> Result<Vec<ClientGroup>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT Id, ProfileId, Name, DisplayOrder, CycleForwardHotkey, CycleBackwardHotkey
                 FROM ClientGroups WHERE ProfileId = ?1 ORDER BY DisplayOrder, Name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([profile_id], |row| {
                Ok(ClientGroup {
                    id: row.get(0)?,
                    profile_id: row.get(1)?,
                    name: row.get(2)?,
                    display_order: row.get(3)?,
                    cycle_forward_hotkey: row.get(4)?,
                    cycle_backward_hotkey: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn update_client_group_hotkeys(
        &self,
        group_id: i64,
        forward_hotkey: String,
        backward_hotkey: String,
    ) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE ClientGroups
             SET CycleForwardHotkey = ?1, CycleBackwardHotkey = ?2
             WHERE Id = ?3",
            params![forward_hotkey, backward_hotkey, group_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_client_group_member_titles(&self, group_id: i64) -> Result<Vec<String>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT WindowTitle FROM ClientGroupMembers
                 WHERE GroupId = ?1
                 ORDER BY DisplayOrder, WindowTitle",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([group_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn get_mumble_links(&self) -> Result<Vec<MumbleLink>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT Id, Name, Url, DisplayOrder, IsSelected, Hotkey FROM MumbleLinks
                 ORDER BY DisplayOrder, Name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(MumbleLink {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    display_order: row.get(3)?,
                    is_selected: row.get::<_, i64>(4)? == 1,
                    hotkey: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn create_mumble_link(
        &self,
        name: String,
        url: String,
        display_order: i64,
        hotkey: String,
    ) -> Result<(), String> {
        validate_link_name(&name)?;
        validate_link_url(&url)?;
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO MumbleLinks (Name, Url, DisplayOrder, IsSelected, Hotkey)
             VALUES (?1, ?2, ?3, 0, ?4)",
            params![name.trim(), url.trim(), display_order, hotkey],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_mumble_link(
        &self,
        link_id: i64,
        name: String,
        url: String,
        display_order: i64,
        hotkey: String,
    ) -> Result<(), String> {
        validate_link_name(&name)?;
        validate_link_url(&url)?;
        let conn = self.connection()?;
        conn.execute(
            "UPDATE MumbleLinks
             SET Name = ?1, Url = ?2, DisplayOrder = ?3, Hotkey = ?4
             WHERE Id = ?5",
            params![name.trim(), url.trim(), display_order, hotkey, link_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_mumble_link_selected(&self, link_id: i64, is_selected: bool) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE MumbleLinks SET IsSelected = ?1 WHERE Id = ?2",
            params![if is_selected { 1 } else { 0 }, link_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_mumble_link(&self, link_id: i64) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM MumbleLinks WHERE Id = ?1", [link_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_mumble_server_groups(&self) -> Result<Vec<MumbleServerGroup>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT Id, Name, DisplayOrder FROM MumbleServerGroups
                 ORDER BY DisplayOrder, Name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(MumbleServerGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_order: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn create_mumble_server_group(&self, name: String, display_order: i64) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Group name cannot be empty".to_string());
        }
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO MumbleServerGroups (Name, DisplayOrder) VALUES (?1, ?2)",
            params![name.trim(), display_order],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_mumble_server_group(
        &self,
        group_id: i64,
        name: String,
        display_order: i64,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Group name cannot be empty".to_string());
        }
        let conn = self.connection()?;
        conn.execute(
            "UPDATE MumbleServerGroups SET Name = ?1, DisplayOrder = ?2 WHERE Id = ?3",
            params![name.trim(), display_order, group_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_mumble_server_group(&self, group_id: i64) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM MumbleServerGroups WHERE Id = ?1", [group_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_mumble_links_overlay_settings(&self) -> Result<MumbleLinksOverlaySettings, String> {
        let conn = self.connection()?;
        let record = conn
            .query_row(
                "SELECT AlwaysOnTop, X, Y, Width, Height
                 FROM MumbleLinksOverlaySettings
                 ORDER BY Id ASC
                 LIMIT 1",
                [],
                |row| {
                    Ok(MumbleLinksOverlaySettings {
                        always_on_top: row.get::<_, i64>(0)? == 1,
                        x: row.get(1)?,
                        y: row.get(2)?,
                        width: row.get(3)?,
                        height: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(value) = record {
            return Ok(value);
        }
        Ok(MumbleLinksOverlaySettings {
            always_on_top: true,
            x: 100,
            y: 100,
            width: 300,
            height: 400,
        })
    }

    pub fn save_mumble_links_overlay_settings(
        &self,
        settings: MumbleLinksOverlaySettings,
    ) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO MumbleLinksOverlaySettings (AlwaysOnTop, X, Y, Width, Height)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                if settings.always_on_top { 1 } else { 0 },
                settings.x,
                settings.y,
                settings.width,
                settings.height
            ],
        )
        .map_err(|e| e.to_string())?;
        let inserted_id = conn.last_insert_rowid();
        conn.execute(
            "DELETE FROM MumbleLinksOverlaySettings WHERE Id <> ?1",
            [inserted_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_drawer_settings(&self) -> Result<DrawerSettings, String> {
        let parse_bool = |s: Option<String>, default_value: bool| -> bool {
            s.map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(default_value)
        };
        let parse_i64 = |s: Option<String>, default_value: i64| -> i64 {
            s.and_then(|v| v.parse::<i64>().ok()).unwrap_or(default_value)
        };
        let parse_opt_i64 = |s: Option<String>| -> Option<i64> {
            let value = s.unwrap_or_default();
            if value.trim().is_empty() {
                None
            } else {
                value.parse::<i64>().ok()
            }
        };

        Ok(DrawerSettings {
            screen_index: parse_i64(self.get_app_setting("DrawerScreenIndex".to_string())?, 0),
            hardware_id: self
                .get_app_setting("DrawerHardwareId".to_string())?
                .unwrap_or_default(),
            side: self
                .get_app_setting("DrawerSide".to_string())?
                .unwrap_or_else(|| "Left".to_string()),
            width: parse_i64(self.get_app_setting("DrawerWidth".to_string())?, 360),
            height: parse_i64(self.get_app_setting("DrawerHeight".to_string())?, 500),
            is_visible: parse_bool(self.get_app_setting("DrawerIsVisible".to_string())?, false),
            is_enabled: parse_bool(self.get_app_setting("DrawerIsEnabled".to_string())?, false),
            selected_mumble_server_group_id: parse_opt_i64(
                self.get_app_setting("DrawerSelectedMumbleServerGroupId".to_string())?,
            ),
        })
    }

    pub fn save_drawer_settings(&self, settings: DrawerSettings) -> Result<(), String> {
        self.set_app_setting("DrawerScreenIndex".to_string(), settings.screen_index.to_string())?;
        self.set_app_setting("DrawerHardwareId".to_string(), settings.hardware_id)?;
        self.set_app_setting("DrawerSide".to_string(), settings.side)?;
        self.set_app_setting("DrawerWidth".to_string(), settings.width.to_string())?;
        self.set_app_setting("DrawerHeight".to_string(), settings.height.to_string())?;
        self.set_app_setting(
            "DrawerIsVisible".to_string(),
            settings.is_visible.to_string(),
        )?;
        self.set_app_setting(
            "DrawerIsEnabled".to_string(),
            settings.is_enabled.to_string(),
        )?;
        self.set_app_setting(
            "DrawerSelectedMumbleServerGroupId".to_string(),
            settings
                .selected_mumble_server_group_id
                .map(|v| v.to_string())
                .unwrap_or_default(),
        )?;
        Ok(())
    }

    pub fn get_app_setting(&self, key: String) -> Result<Option<String>, String> {
        let conn = self.connection()?;
        conn.query_row("SELECT Value FROM AppSettings WHERE Key = ?1", [key], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn set_app_setting(&self, key: String, value: String) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO AppSettings (Key, Value) VALUES (?1, ?2)
             ON CONFLICT(Key) DO UPDATE SET Value = excluded.Value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn validate_link_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Link name cannot be empty".to_string());
    }
    Ok(())
}

fn validate_link_url(url: &str) -> Result<(), String> {
    let value = url.trim();
    if value.is_empty() {
        return Err("Link URL cannot be empty".to_string());
    }
    let allowed = value.starts_with("mumble://")
        || value.starts_with("https://")
        || value.starts_with("http://");
    if !allowed {
        return Err("Link URL must start with mumble://, https://, or http://".to_string());
    }
    Ok(())
}
