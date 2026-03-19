use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{
    ClientGroup, MumbleLink, MumbleServerGroup, Profile, ThumbnailConfig, ThumbnailSetting,
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
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM ProcessesToPreview WHERE ProfileId = ?1 AND ProcessName = ?2",
            params![profile_id, process_name.trim().to_lowercase()],
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
