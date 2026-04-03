use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::models::{
    ClientGroup, ClientGroupDetail, ClientGroupMember, DrawerSettings, EveChatChannel,
    EveLogSettings, GridLayoutFormPrefs, MumbleFolder, MumbleLink, MumbleLinksOverlaySettings,
    MumbleServerGroup, MumbleTreeSnapshot, Profile, ThumbnailConfig, ThumbnailSetting,
};

const GRID_LAYOUT_PREFS_KEY: &str = "GridLayoutPrefsJson";

#[derive(Debug, Serialize, Deserialize, Default)]
struct GridLayoutPrefsStore {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    by_profile: HashMap<String, GridLayoutFormPrefs>,
}

pub struct DbService {
    db_path: PathBuf,
}

fn normalize_color_hex(input: Option<&str>) -> String {
    let default_color = "#1f2937".to_string();
    let Some(raw) = input else {
        return default_color;
    };
    let trimmed = raw.trim();
    let value = if let Some(v) = trimmed.strip_prefix('#') {
        v
    } else {
        trimmed
    };
    if value.len() != 6 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return default_color;
    }
    format!("#{}", value.to_ascii_lowercase())
}

impl DbService {
    /// SQLite settings file used at runtime. Log with [`crate::diag::trace`] when debugging import/export (YAEP_DIAG).
    pub fn settings_db_path(&self) -> &Path {
        self.db_path.as_path()
    }

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

    pub(crate) fn db_conn(&self) -> Result<Connection, String> {
        self.connection()
    }

    fn initialize(&self) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute_batch(include_str!("../sql/bootstrap.sql"))
            .map_err(|e| e.to_string())?;
        conn.execute_batch(include_str!("../sql/mumble_and_groups.sql"))
            .map_err(|e| e.to_string())?;
        conn.execute_batch(include_str!("../sql/mumble_folders.sql"))
            .map_err(|e| e.to_string())?;
        self.ensure_mumble_links_tree_columns(&conn)?;
        self.ensure_mumble_folder_icon_key_column(&conn)?;
        self.ensure_thumbnail_character_id_column(&conn)?;
        self.ensure_thumbnail_decloak_flash_columns(&conn)?;
        self.ensure_eve_chat_channel_background_color_column(&conn)?;
        self.bootstrap_defaults(&conn)
    }

    fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| e.to_string())?;
        let cols = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        for c in cols {
            if c.map_err(|e| e.to_string())?.eq_ignore_ascii_case(column) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn ensure_mumble_folder_icon_key_column(&self, conn: &Connection) -> Result<(), String> {
        if !Self::table_has_column(conn, "MumbleFolders", "IconKey")? {
            conn.execute(
                "ALTER TABLE MumbleFolders ADD COLUMN IconKey TEXT NULL",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn ensure_mumble_links_tree_columns(&self, conn: &Connection) -> Result<(), String> {
        if !Self::table_has_column(conn, "MumbleLinks", "ServerGroupId")? {
            conn.execute(
                "ALTER TABLE MumbleLinks ADD COLUMN ServerGroupId INTEGER NULL",
                [],
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "ALTER TABLE MumbleLinks ADD COLUMN FolderId INTEGER NULL",
                [],
            )
            .map_err(|e| e.to_string())?;
        } else if !Self::table_has_column(conn, "MumbleLinks", "FolderId")? {
            conn.execute(
                "ALTER TABLE MumbleLinks ADD COLUMN FolderId INTEGER NULL",
                [],
            )
            .map_err(|e| e.to_string())?;
        }

        let group_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM MumbleServerGroups", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if group_count == 0 {
            conn.execute(
                "INSERT INTO MumbleServerGroups (Name, DisplayOrder) VALUES ('Default', 0)",
                [],
            )
            .map_err(|e| e.to_string())?;
        }

        let default_gid: i64 = conn
            .query_row(
                "SELECT Id FROM MumbleServerGroups ORDER BY DisplayOrder ASC, Id ASC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE MumbleLinks
             SET ServerGroupId = (
               SELECT GroupId FROM MumbleLinkGroups
               WHERE MumbleLinkGroups.LinkId = MumbleLinks.Id LIMIT 1
             )
             WHERE ServerGroupId IS NULL
               AND EXISTS (
                 SELECT 1 FROM MumbleLinkGroups ml
                 WHERE ml.LinkId = MumbleLinks.Id
               )",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE MumbleLinks SET ServerGroupId = ?1 WHERE ServerGroupId IS NULL",
            params![default_gid],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn ensure_thumbnail_character_id_column(&self, conn: &Connection) -> Result<(), String> {
        let mut stmt = conn
            .prepare("PRAGMA table_info(ThumbnailSettings)")
            .map_err(|e| e.to_string())?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        for col in columns {
            if col.map_err(|e| e.to_string())?.eq_ignore_ascii_case("CharacterId") {
                return Ok(());
            }
        }
        conn.execute("ALTER TABLE ThumbnailSettings ADD COLUMN CharacterId INTEGER NULL", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn ensure_eve_chat_channel_background_color_column(
        &self,
        conn: &Connection,
    ) -> Result<(), String> {
        let mut stmt = conn
            .prepare("PRAGMA table_info(EveChatChannels)")
            .map_err(|e| e.to_string())?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        for col in columns {
            if col
                .map_err(|e| e.to_string())?
                .eq_ignore_ascii_case("BackgroundColor")
            {
                return Ok(());
            }
        }
        conn.execute(
            "ALTER TABLE EveChatChannels ADD COLUMN BackgroundColor TEXT NOT NULL DEFAULT '#1f2937'",
            [],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn ensure_thumbnail_decloak_flash_columns(&self, conn: &Connection) -> Result<(), String> {
        let mut default_stmt = conn
            .prepare("PRAGMA table_info(ThumbnailDefaultConfig)")
            .map_err(|e| e.to_string())?;
        let default_columns = default_stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        let mut has_default_color = false;
        let mut has_default_thickness = false;
        let mut has_default_duration = false;
        for col in default_columns {
            let col = col.map_err(|e| e.to_string())?;
            if col.eq_ignore_ascii_case("DecloakFlashColor") {
                has_default_color = true;
            } else if col.eq_ignore_ascii_case("DecloakFlashThickness") {
                has_default_thickness = true;
            } else if col.eq_ignore_ascii_case("DecloakFlashDurationMs") {
                has_default_duration = true;
            }
        }
        if !has_default_color {
            conn.execute(
                "ALTER TABLE ThumbnailDefaultConfig ADD COLUMN DecloakFlashColor TEXT NOT NULL DEFAULT '#fcd34d'",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        if !has_default_thickness {
            conn.execute(
                "ALTER TABLE ThumbnailDefaultConfig ADD COLUMN DecloakFlashThickness INTEGER NOT NULL DEFAULT 2",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        if !has_default_duration {
            conn.execute(
                "ALTER TABLE ThumbnailDefaultConfig ADD COLUMN DecloakFlashDurationMs INTEGER NOT NULL DEFAULT 5000",
                [],
            )
            .map_err(|e| e.to_string())?;
        }

        let mut settings_stmt = conn
            .prepare("PRAGMA table_info(ThumbnailSettings)")
            .map_err(|e| e.to_string())?;
        let settings_columns = settings_stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        let mut has_settings_color = false;
        let mut has_settings_thickness = false;
        let mut has_settings_duration = false;
        for col in settings_columns {
            let col = col.map_err(|e| e.to_string())?;
            if col.eq_ignore_ascii_case("DecloakFlashColor") {
                has_settings_color = true;
            } else if col.eq_ignore_ascii_case("DecloakFlashThickness") {
                has_settings_thickness = true;
            } else if col.eq_ignore_ascii_case("DecloakFlashDurationMs") {
                has_settings_duration = true;
            }
        }
        if !has_settings_color {
            conn.execute(
                "ALTER TABLE ThumbnailSettings ADD COLUMN DecloakFlashColor TEXT NOT NULL DEFAULT '#fcd34d'",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        if !has_settings_thickness {
            conn.execute(
                "ALTER TABLE ThumbnailSettings ADD COLUMN DecloakFlashThickness INTEGER NOT NULL DEFAULT 2",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        if !has_settings_duration {
            conn.execute(
                "ALTER TABLE ThumbnailSettings ADD COLUMN DecloakFlashDurationMs INTEGER NOT NULL DEFAULT 5000",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
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
            (ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay)
            VALUES (?1, 400, 300, 100, 100, 0.75, '#0078D4', 3, '#fcd34d', 2, 5000, 1)",
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
            (ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay)
            VALUES (?1, 400, 300, 100, 100, 0.75, '#0078D4', 3, '#fcd34d', 2, 5000, 1)",
            [profile_id],
        )
        .map_err(|e| e.to_string())?;
        let next_cg: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(DisplayOrder), -1) + 1 FROM ClientGroups WHERE ProfileId = ?1",
                [profile_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO ClientGroups (ProfileId, Name, DisplayOrder, CycleForwardHotkey, CycleBackwardHotkey)
             VALUES (?1, 'Default', ?2, '', '')",
            params![profile_id, next_cg],
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
            .query_row(
                "SELECT Id FROM Profile WHERE Id = ?1 LIMIT 1",
                [profile_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if exists.is_none() {
            return Err("Profile not found".to_string());
        }
        conn.execute("UPDATE Profile SET IsActive = 0", [])
            .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE Profile SET IsActive = 1 WHERE Id = ?1",
            [profile_id],
        )
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
        conn.query_row(
            "SELECT Id FROM Profile WHERE IsActive = 1 LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten()
    }

    fn default_chat_logs_path() -> String {
        let user_profile = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        format!("{user_profile}\\Documents\\EVE\\logs\\Chatlogs")
    }

    fn default_game_logs_path() -> String {
        let user_profile = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        format!("{user_profile}\\Documents\\EVE\\logs\\Gamelogs")
    }

    fn normalize_required_path(value: &str, label: &str) -> Result<String, String> {
        let normalized = value.trim().replace('/', "\\");
        if normalized.is_empty() {
            return Err(format!("{label} cannot be empty"));
        }
        Ok(normalized)
    }

    pub fn get_eve_log_settings(&self, profile_id: i64) -> Result<EveLogSettings, String> {
        let conn = self.connection()?;
        let row: Option<EveLogSettings> = conn
            .query_row(
                "SELECT ChatLogsPath, GameLogsPath FROM EveLogSettings WHERE ProfileId = ?1",
                [profile_id],
                |row| {
                    Ok(EveLogSettings {
                        chat_logs_path: row.get(0)?,
                        game_logs_path: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(row.unwrap_or(EveLogSettings {
            chat_logs_path: Self::default_chat_logs_path(),
            game_logs_path: Self::default_game_logs_path(),
        }))
    }

    pub fn save_eve_log_settings(&self, profile_id: i64, settings: EveLogSettings) -> Result<(), String> {
        let conn = self.connection()?;
        let chat_logs_path = Self::normalize_required_path(&settings.chat_logs_path, "Chat logs path")?;
        let game_logs_path = Self::normalize_required_path(&settings.game_logs_path, "Game logs path")?;
        conn.execute(
            "INSERT INTO EveLogSettings (ProfileId, ChatLogsPath, GameLogsPath)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(ProfileId) DO UPDATE SET
               ChatLogsPath = excluded.ChatLogsPath,
               GameLogsPath = excluded.GameLogsPath",
            params![profile_id, chat_logs_path, game_logs_path],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_eve_chat_channels(&self, profile_id: i64) -> Result<Vec<EveChatChannel>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT Id, ProfileId, ChannelType, ChannelName
                 , BackgroundColor
                 FROM EveChatChannels
                 WHERE ProfileId = ?1
                 ORDER BY ChannelType, ChannelName",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([profile_id], |row| {
                Ok(EveChatChannel {
                    id: row.get(0)?,
                    profile_id: row.get(1)?,
                    channel_type: row.get(2)?,
                    channel_name: row.get(3)?,
                    background_color: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn add_eve_chat_channel(
        &self,
        profile_id: i64,
        channel_type: String,
        channel_name: String,
        background_color: Option<String>,
    ) -> Result<EveChatChannel, String> {
        let conn = self.connection()?;
        let normalized_type = channel_type.trim();
        if normalized_type != "FleetBoost" && normalized_type != "Intel" {
            return Err("Channel type must be FleetBoost or Intel".to_string());
        }
        let normalized_name = channel_name.trim();
        if normalized_name.is_empty() {
            return Err("Channel name cannot be empty".to_string());
        }
        let duplicate_exists: Option<i64> = conn
            .query_row(
                "SELECT Id
                 FROM EveChatChannels
                 WHERE ProfileId = ?1 AND lower(trim(ChannelName)) = lower(trim(?2))
                 LIMIT 1",
                params![profile_id, normalized_name],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if duplicate_exists.is_some() {
            return Err("Channel name already exists".to_string());
        }
        let normalized_background_color = normalize_color_hex(background_color.as_deref());
        conn.execute(
            "INSERT INTO EveChatChannels (ProfileId, ChannelType, ChannelName, BackgroundColor) VALUES (?1, ?2, ?3, ?4)",
            params![profile_id, normalized_type, normalized_name, normalized_background_color],
        )
        .map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        Ok(EveChatChannel {
            id,
            profile_id,
            channel_type: normalized_type.to_string(),
            channel_name: normalized_name.to_string(),
            background_color: normalized_background_color,
        })
    }

    pub fn update_eve_chat_channel_color(
        &self,
        profile_id: i64,
        channel_id: i64,
        background_color: String,
    ) -> Result<(), String> {
        let conn = self.connection()?;
        let normalized = normalize_color_hex(Some(&background_color));
        conn.execute(
            "UPDATE EveChatChannels
             SET BackgroundColor = ?3
             WHERE ProfileId = ?1 AND Id = ?2",
            params![profile_id, channel_id, normalized],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_eve_chat_channel(&self, profile_id: i64, channel_id: i64) -> Result<(), String> {
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM EveChatChannels WHERE ProfileId = ?1 AND Id = ?2",
            params![profile_id, channel_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
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

    pub fn add_process_to_preview(
        &self,
        profile_id: i64,
        process_name: String,
    ) -> Result<(), String> {
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

    pub fn remove_process_to_preview(
        &self,
        profile_id: i64,
        process_name: String,
    ) -> Result<(), String> {
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
            "SELECT Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay
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
                    decloak_flash_color: row.get(7)?,
                    decloak_flash_thickness: row.get(8)?,
                    decloak_flash_duration_ms: row.get(9)?,
                    show_title_overlay: row.get::<_, i64>(10)? == 1,
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
            (ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(ProfileId) DO UPDATE SET
              Width=excluded.Width,
              Height=excluded.Height,
              X=excluded.X,
              Y=excluded.Y,
              Opacity=excluded.Opacity,
              FocusBorderColor=excluded.FocusBorderColor,
              FocusBorderThickness=excluded.FocusBorderThickness,
              DecloakFlashColor=excluded.DecloakFlashColor,
              DecloakFlashThickness=excluded.DecloakFlashThickness,
              DecloakFlashDurationMs=excluded.DecloakFlashDurationMs,
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
                config.decloak_flash_color,
                config.decloak_flash_thickness,
                config.decloak_flash_duration_ms,
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
                 DecloakFlashColor = ?7,
                 DecloakFlashThickness = ?8,
                 DecloakFlashDurationMs = ?9,
                 ShowTitleOverlay = ?10
             WHERE ProfileId = ?1",
            params![
                profile_id,
                config.width,
                config.height,
                config.opacity,
                config.focus_border_color,
                config.focus_border_thickness,
                config.decloak_flash_color,
                config.decloak_flash_thickness,
                config.decloak_flash_duration_ms,
                if config.show_title_overlay { 1 } else { 0 }
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn thumbnail_config_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ThumbnailConfig> {
        Ok(ThumbnailConfig {
            width: row.get(0)?,
            height: row.get(1)?,
            x: row.get(2)?,
            y: row.get(3)?,
            opacity: row.get(4)?,
            focus_border_color: row.get(5)?,
            focus_border_thickness: row.get(6)?,
            show_title_overlay: row.get::<_, i64>(7)? == 1,
            decloak_flash_color: row.get(8)?,
            decloak_flash_thickness: row.get(9)?,
            decloak_flash_duration_ms: row.get(10)?,
        })
    }

    /// Load saved thumbnail config for this profile using **trimmed** window title equality so keys
    /// match the OS title even if spacing differs.
    pub fn get_thumbnail_config_trimmed(
        &self,
        profile_id: i64,
        window_title: &str,
    ) -> Result<Option<ThumbnailConfig>, String> {
        let t = window_title.trim();
        if t.is_empty() {
            return Ok(None);
        }
        let conn = self.connection()?;
        let row = conn
            .query_row(
                "SELECT Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay
                 , DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs
                 FROM ThumbnailSettings WHERE ProfileId = ?1 AND TRIM(WindowTitle) = ?2",
                params![profile_id, t],
                Self::thumbnail_config_from_row,
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if row.is_some() {
            return Ok(row);
        }
        let needle = t.to_lowercase();
        let row_ci = conn
            .query_row(
                "SELECT Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay
                 , DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs
                 FROM ThumbnailSettings WHERE ProfileId = ?1 AND LOWER(TRIM(WindowTitle)) = ?2
                 LIMIT 1",
                params![profile_id, needle],
                Self::thumbnail_config_from_row,
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(row_ci)
    }

    /// Renames persisted thumbnail + client-group rows when the same process gets a new window title
    /// so X/Y and other settings stay with the client.
    pub fn migrate_thumbnail_window_title(
        &self,
        profile_id: i64,
        from_title: &str,
        to_title: &str,
    ) -> Result<(), String> {
        let from = from_title.trim();
        let to = to_title.trim();
        if from.is_empty() || to.is_empty() || from == to {
            return Ok(());
        }
        let mut conn = self.connection()?;
        let dest_exists: bool = conn
            .query_row(
                "SELECT 1 FROM ThumbnailSettings WHERE ProfileId = ?1 AND TRIM(WindowTitle) = ?2 LIMIT 1",
                params![profile_id, to],
                |_| Ok(()),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .is_some();
        if dest_exists {
            return Ok(());
        }
        let has_source: bool = conn
            .query_row(
                "SELECT 1 FROM ThumbnailSettings WHERE ProfileId = ?1 AND TRIM(WindowTitle) = ?2 LIMIT 1",
                params![profile_id, from],
                |_| Ok(()),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .is_some();
        if !has_source {
            return Ok(());
        }
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let n = tx
            .execute(
                "UPDATE ThumbnailSettings SET WindowTitle = ?1 WHERE ProfileId = ?2 AND TRIM(WindowTitle) = ?3",
                params![to, profile_id, from],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            tx.rollback().map_err(|e| e.to_string())?;
            return Ok(());
        }
        tx.execute(
            "UPDATE ClientGroupMembers SET WindowTitle = ?1
             WHERE TRIM(WindowTitle) = ?2 AND GroupId IN (SELECT Id FROM ClientGroups WHERE ProfileId = ?3)",
            params![to, from, profile_id],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Same as [`Self::resolve_thumbnail_config`], but when the window title changes for an existing
    /// PID, reuses and migrates settings from the prior title so coordinates do not reset.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub fn resolve_thumbnail_config_for_registration(
        &self,
        profile_id: i64,
        window_title: &str,
        same_pid_prior_title: Option<&str>,
    ) -> Result<ThumbnailConfig, String> {
        let t = window_title.trim();
        if let Some(c) = self.get_thumbnail_config_trimmed(profile_id, t)? {
            return Ok(c);
        }
        if let Some(prior) = same_pid_prior_title {
            let p = prior.trim();
            if !p.is_empty() && p != t {
                if self.get_thumbnail_config_trimmed(profile_id, p)?.is_some() {
                    self.migrate_thumbnail_window_title(profile_id, p, t)?;
                    if let Some(c) = self.get_thumbnail_config_trimmed(profile_id, t)? {
                        return Ok(c);
                    }
                }
            }
        }
        let mut config = self.get_thumbnail_default_config(profile_id)?;
        self.apply_default_client_group_slot_x(profile_id, t, &mut config)?;
        Ok(config)
    }

    /// Per-window override if present (trimmed title), otherwise profile default + optional slot X.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub fn resolve_thumbnail_config(
        &self,
        profile_id: i64,
        window_title: &str,
    ) -> Result<ThumbnailConfig, String> {
        if let Some(config) = self.get_thumbnail_config_trimmed(profile_id, window_title)? {
            return Ok(config);
        }
        let mut config = self.get_thumbnail_default_config(profile_id)?;
        self.apply_default_client_group_slot_x(profile_id, window_title.trim(), &mut config)?;
        Ok(config)
    }

    /// Horizontal slot for a window title in the profile's **Default** client group (member display
    /// order). New thumbnails with no `ThumbnailSettings` row yet get `X` offset from the profile
    /// default origin so they line up with group order. If the title is not in the group yet (first
    /// registration), uses the index after current members (append).
    fn apply_default_client_group_slot_x(
        &self,
        profile_id: i64,
        window_title: &str,
        config: &mut ThumbnailConfig,
    ) -> Result<(), String> {
        const GAP: i64 = 12;
        let conn = self.connection()?;
        let default_group_id: Option<i64> = conn
            .query_row(
                "SELECT Id FROM ClientGroups WHERE ProfileId = ?1 AND Name = 'Default' LIMIT 1",
                [profile_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some(group_id) = default_group_id else {
            return Ok(());
        };
        let titles = self.get_client_group_member_titles(group_id)?;
        let t = window_title.trim();
        let slot = titles
            .iter()
            .position(|x| x.trim() == t)
            .unwrap_or(titles.len());
        let step = config.width.saturating_add(GAP);
        config.x = config
            .x
            .saturating_add((slot as i64).saturating_mul(step));
        Ok(())
    }

    pub fn get_thumbnail_settings(&self, profile_id: i64) -> Result<Vec<ThumbnailSetting>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT WindowTitle, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, ShowTitleOverlay, CharacterId
                 , DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs
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
                        decloak_flash_color: row.get(10)?,
                        decloak_flash_thickness: row.get(11)?,
                        decloak_flash_duration_ms: row.get(12)?,
                    },
                    character_id: row.get(9)?,
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
        character_id: Option<i64>,
    ) -> Result<(), String> {
        let window_title = window_title.trim().to_string();
        if window_title.is_empty() {
            return Err("Window title cannot be empty".to_string());
        }
        let conn = self.connection()?;
        let was_new_thumbnail: bool = conn
            .query_row(
                "SELECT 1 FROM ThumbnailSettings WHERE ProfileId = ?1 AND TRIM(WindowTitle) = ?2 LIMIT 1",
                params![profile_id, &window_title],
                |_| Ok(()),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .is_none();
        conn.execute(
            "INSERT INTO ThumbnailSettings
             (ProfileId, WindowTitle, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness, DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay, CharacterId)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(ProfileId, WindowTitle) DO UPDATE SET
               Width=excluded.Width, Height=excluded.Height, X=excluded.X, Y=excluded.Y,
               Opacity=excluded.Opacity, FocusBorderColor=excluded.FocusBorderColor,
               FocusBorderThickness=excluded.FocusBorderThickness,
               DecloakFlashColor=excluded.DecloakFlashColor, DecloakFlashThickness=excluded.DecloakFlashThickness,
               DecloakFlashDurationMs=excluded.DecloakFlashDurationMs, ShowTitleOverlay=excluded.ShowTitleOverlay,
               CharacterId=COALESCE(excluded.CharacterId, ThumbnailSettings.CharacterId)",
            params![
                profile_id,
                &window_title,
                config.width,
                config.height,
                config.x,
                config.y,
                config.opacity,
                config.focus_border_color,
                config.focus_border_thickness,
                config.decloak_flash_color,
                config.decloak_flash_thickness,
                config.decloak_flash_duration_ms,
                if config.show_title_overlay { 1 } else { 0 },
                character_id
            ],
        )
        .map_err(|e| e.to_string())?;

        if was_new_thumbnail {
            let default_group_id: Option<i64> = conn
                .query_row(
                    "SELECT Id FROM ClientGroups WHERE ProfileId = ?1 AND Name = 'Default' LIMIT 1",
                    [profile_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            if let Some(group_id) = default_group_id {
                let next_order: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(DisplayOrder), -1) + 1
                         FROM ClientGroupMembers WHERE GroupId = ?1",
                        [group_id],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                conn.execute(
                    "INSERT OR IGNORE INTO ClientGroupMembers (GroupId, WindowTitle, DisplayOrder)
                     VALUES (?1, ?2, ?3)",
                    params![group_id, &window_title, next_order],
                )
                .map_err(|e| e.to_string())?;
            }
        }

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

    pub fn ensure_client_group_profile(
        &self,
        group_id: i64,
        profile_id: i64,
    ) -> Result<(), String> {
        let conn = self.connection()?;
        let pid: Option<i64> = conn
            .query_row(
                "SELECT ProfileId FROM ClientGroups WHERE Id = ?1",
                [group_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        match pid {
            None => Err("Client group not found".to_string()),
            Some(p) if p != profile_id => {
                Err("Client group does not belong to this profile".to_string())
            }
            Some(_) => Ok(()),
        }
    }

    pub fn get_client_group_members(
        &self,
        group_id: i64,
    ) -> Result<Vec<ClientGroupMember>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT WindowTitle, DisplayOrder FROM ClientGroupMembers
                 WHERE GroupId = ?1
                 ORDER BY DisplayOrder, WindowTitle",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([group_id], |row| {
                Ok(ClientGroupMember {
                    window_title: row.get(0)?,
                    display_order: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn get_client_group_member_titles(&self, group_id: i64) -> Result<Vec<String>, String> {
        Ok(self
            .get_client_group_members(group_id)?
            .into_iter()
            .map(|m| m.window_title)
            .collect())
    }

    /// Flattened member order for grid layout when no single group is selected: walk client groups
    /// by display order, then members by display order; each trimmed title appears at most once
    /// (first group wins if a title appears in multiple groups).
    pub fn client_group_combined_member_order(&self, profile_id: i64) -> Result<Vec<String>, String> {
        let groups = self.get_client_groups(profile_id)?;
        let mut seen = HashSet::<String>::new();
        let mut order = Vec::new();
        for g in groups {
            let members = self.get_client_group_members(g.id)?;
            for m in members {
                let t = m.window_title.trim().to_string();
                if t.is_empty() || seen.contains(&t) {
                    continue;
                }
                seen.insert(t.clone());
                order.push(t);
            }
        }
        Ok(order)
    }

    pub fn get_client_groups_detailed(
        &self,
        profile_id: i64,
    ) -> Result<Vec<ClientGroupDetail>, String> {
        let groups = self.get_client_groups(profile_id)?;
        let mut out = Vec::new();
        for g in groups {
            let members = self.get_client_group_members(g.id)?;
            out.push(ClientGroupDetail {
                id: g.id,
                profile_id: g.profile_id,
                name: g.name,
                display_order: g.display_order,
                cycle_forward_hotkey: g.cycle_forward_hotkey,
                cycle_backward_hotkey: g.cycle_backward_hotkey,
                members,
            });
        }
        Ok(out)
    }

    pub fn create_client_group(
        &self,
        profile_id: i64,
        name: String,
    ) -> Result<ClientGroupDetail, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Group name cannot be empty".to_string());
        }
        let conn = self.connection()?;
        let exists: Option<i64> = conn
            .query_row(
                "SELECT Id FROM ClientGroups WHERE ProfileId = ?1 AND Name = ?2 LIMIT 1",
                params![profile_id, trimmed],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if exists.is_some() {
            return Err("A client group with this name already exists".to_string());
        }
        let next_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(DisplayOrder), -1) + 1 FROM ClientGroups WHERE ProfileId = ?1",
                [profile_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO ClientGroups (ProfileId, Name, DisplayOrder, CycleForwardHotkey, CycleBackwardHotkey)
             VALUES (?1, ?2, ?3, '', '')",
            params![profile_id, trimmed, next_order],
        )
        .map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        Ok(ClientGroupDetail {
            id,
            profile_id,
            name: trimmed.to_string(),
            display_order: next_order,
            cycle_forward_hotkey: String::new(),
            cycle_backward_hotkey: String::new(),
            members: Vec::new(),
        })
    }

    pub fn delete_client_group(&self, profile_id: i64, group_id: i64) -> Result<(), String> {
        self.ensure_client_group_profile(group_id, profile_id)?;
        let conn = self.connection()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ClientGroups WHERE ProfileId = ?1",
                [profile_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if count <= 1 {
            return Err("Cannot delete the last client group".to_string());
        }
        conn.execute("DELETE FROM ClientGroups WHERE Id = ?1", [group_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_client_group_member(
        &self,
        profile_id: i64,
        group_id: i64,
        window_title: String,
    ) -> Result<(), String> {
        self.ensure_client_group_profile(group_id, profile_id)?;
        let title = window_title.trim();
        if title.is_empty() {
            return Err("Window title cannot be empty".to_string());
        }
        let conn = self.connection()?;
        let next_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(DisplayOrder), -1) + 1 FROM ClientGroupMembers WHERE GroupId = ?1",
                [group_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO ClientGroupMembers (GroupId, WindowTitle, DisplayOrder) VALUES (?1, ?2, ?3)",
            params![group_id, title, next_order],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                "This client is already in the group".to_string()
            } else {
                e.to_string()
            }
        })?;
        Ok(())
    }

    pub fn remove_client_group_member(
        &self,
        profile_id: i64,
        group_id: i64,
        window_title: String,
    ) -> Result<(), String> {
        self.ensure_client_group_profile(group_id, profile_id)?;
        let conn = self.connection()?;
        conn.execute(
            "DELETE FROM ClientGroupMembers WHERE GroupId = ?1 AND WindowTitle = ?2",
            params![group_id, window_title.trim()],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reorder_client_group_members(
        &self,
        profile_id: i64,
        group_id: i64,
        window_titles_in_order: Vec<String>,
    ) -> Result<(), String> {
        self.ensure_client_group_profile(group_id, profile_id)?;
        let mut conn = self.connection()?;
        let current: HashSet<String> = {
            let mut stmt = conn
                .prepare("SELECT WindowTitle FROM ClientGroupMembers WHERE GroupId = ?1")
                .map_err(|e| e.to_string())?;
            let titles: HashSet<String> = stmt
                .query_map([group_id], |row| row.get::<_, String>(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?;
            titles
        };
        let ordered: Vec<String> = window_titles_in_order
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        let new_set: HashSet<String> = ordered.iter().cloned().collect();
        if new_set.len() != ordered.len() {
            return Err("Duplicate window titles in order list".to_string());
        }
        if current != new_set {
            return Err("Order list must contain exactly the current group members".to_string());
        }
        if current.len() != ordered.len() {
            return Err("Order list must contain exactly the current group members".to_string());
        }
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for (i, title) in ordered.iter().enumerate() {
            tx.execute(
                "UPDATE ClientGroupMembers SET DisplayOrder = ?1 WHERE GroupId = ?2 AND WindowTitle = ?3",
                params![i as i64, group_id, title],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_mumble_links(&self) -> Result<Vec<MumbleLink>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT Id, Name, Url, DisplayOrder, IsSelected, Hotkey, ServerGroupId, FolderId
                 FROM MumbleLinks
                 ORDER BY ServerGroupId, FolderId NULLS FIRST, DisplayOrder, Name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| Self::mumble_link_from_row(row))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    fn mumble_link_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MumbleLink> {
        let folder_id: Option<i64> = row.get(7)?;
        Ok(MumbleLink {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            display_order: row.get(3)?,
            is_selected: row.get::<_, i64>(4)? == 1,
            hotkey: row.get(5)?,
            server_group_id: row.get(6)?,
            folder_id,
        })
    }

    pub fn get_mumble_tree(&self) -> Result<MumbleTreeSnapshot, String> {
        let groups = self.get_mumble_server_groups()?;
        let folders = self.get_mumble_folders()?;
        let links = self.get_mumble_links()?;
        Ok(MumbleTreeSnapshot {
            groups,
            folders,
            links,
        })
    }

    fn normalize_mumble_folder_icon_key(raw: Option<String>) -> Result<Option<String>, String> {
        match raw {
            None => Ok(None),
            Some(s) => {
                let t = s.trim().to_ascii_lowercase();
                if t.is_empty() {
                    return Ok(None);
                }
                if t.len() > 48 {
                    return Err("Folder icon key is too long".to_string());
                }
                if !t
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                {
                    return Err(
                        "Folder icon key must use lowercase letters, digits, and hyphens only"
                            .to_string(),
                    );
                }
                Ok(Some(t))
            }
        }
    }

    fn get_mumble_folders(&self) -> Result<Vec<MumbleFolder>, String> {
        let conn = self.connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT Id, ServerGroupId, ParentFolderId, Name, DisplayOrder, IconKey
                 FROM MumbleFolders
                 ORDER BY ServerGroupId, ParentFolderId NULLS FIRST, DisplayOrder, Name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(MumbleFolder {
                    id: row.get(0)?,
                    server_group_id: row.get(1)?,
                    parent_folder_id: row.get(2)?,
                    name: row.get(3)?,
                    display_order: row.get(4)?,
                    icon_key: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    fn mumble_server_group_exists(&self, conn: &Connection, group_id: i64) -> Result<bool, String> {
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM MumbleServerGroups WHERE Id = ?1",
                params![group_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(n > 0)
    }

    fn mumble_folder_row(
        &self,
        conn: &Connection,
        folder_id: i64,
    ) -> Result<Option<(i64, Option<i64>)>, String> {
        conn.query_row(
            "SELECT ServerGroupId, ParentFolderId FROM MumbleFolders WHERE Id = ?1",
            params![folder_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    fn validate_mumble_link_placement(
        &self,
        conn: &Connection,
        server_group_id: i64,
        folder_id: Option<i64>,
    ) -> Result<(), String> {
        if !self.mumble_server_group_exists(conn, server_group_id)? {
            return Err("Server group not found".to_string());
        }
        if let Some(fid) = folder_id {
            let Some((gid, _)) = self.mumble_folder_row(conn, fid)? else {
                return Err("Folder not found".to_string());
            };
            if gid != server_group_id {
                return Err("Folder does not belong to this server group".to_string());
            }
        }
        Ok(())
    }

    pub fn create_mumble_folder(
        &self,
        server_group_id: i64,
        parent_folder_id: Option<i64>,
        name: String,
        display_order: i64,
        icon_key: Option<String>,
    ) -> Result<i64, String> {
        if name.trim().is_empty() {
            return Err("Folder name cannot be empty".to_string());
        }
        let icon_key = Self::normalize_mumble_folder_icon_key(icon_key)?;
        let conn = self.connection()?;
        if !self.mumble_server_group_exists(&conn, server_group_id)? {
            return Err("Server group not found".to_string());
        }
        if let Some(pid) = parent_folder_id {
            let Some((gid, _)) = self.mumble_folder_row(&conn, pid)? else {
                return Err("Parent folder not found".to_string());
            };
            if gid != server_group_id {
                return Err("Parent folder does not belong to this server group".to_string());
            }
        }
        conn.execute(
            "INSERT INTO MumbleFolders (ServerGroupId, ParentFolderId, Name, DisplayOrder, IconKey)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                server_group_id,
                parent_folder_id,
                name.trim(),
                display_order,
                icon_key
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_mumble_folder(
        &self,
        folder_id: i64,
        name: String,
        display_order: i64,
        icon_key: Option<String>,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Folder name cannot be empty".to_string());
        }
        let icon_key = Self::normalize_mumble_folder_icon_key(icon_key)?;
        let conn = self.connection()?;
        let n = conn
            .execute(
                "UPDATE MumbleFolders SET Name = ?1, DisplayOrder = ?2, IconKey = ?3 WHERE Id = ?4",
                params![name.trim(), display_order, icon_key, folder_id],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Folder not found".to_string());
        }
        Ok(())
    }

    fn delete_mumble_folder_cascade(&self, conn: &Connection, folder_id: i64) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT Id FROM MumbleFolders WHERE ParentFolderId = ?1")
            .map_err(|e| e.to_string())?;
        let children: Vec<i64> = stmt
            .query_map(params![folder_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for cid in children {
            self.delete_mumble_folder_cascade(conn, cid)?;
        }
        conn.execute(
            "DELETE FROM MumbleLinks WHERE FolderId = ?1",
            params![folder_id],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM MumbleFolders WHERE Id = ?1", params![folder_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_mumble_folder(&self, folder_id: i64) -> Result<(), String> {
        let conn = self.connection()?;
        self.delete_mumble_folder_cascade(&conn, folder_id)
    }

    fn delete_mumble_folders_for_server(
        &self,
        conn: &Connection,
        server_group_id: i64,
    ) -> Result<(), String> {
        loop {
            let n = conn
                .execute(
                    "DELETE FROM MumbleFolders
                     WHERE ServerGroupId = ?1
                       AND Id NOT IN (
                         SELECT ParentFolderId FROM MumbleFolders
                         WHERE ParentFolderId IS NOT NULL
                       )",
                    params![server_group_id],
                )
                .map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
        }
        Ok(())
    }

    pub fn create_mumble_link(
        &self,
        name: String,
        url: String,
        display_order: i64,
        hotkey: String,
        server_group_id: i64,
        folder_id: Option<i64>,
    ) -> Result<(), String> {
        validate_link_name(&name)?;
        validate_link_url(&url)?;
        let conn = self.connection()?;
        self.validate_mumble_link_placement(&conn, server_group_id, folder_id)?;
        conn.execute(
            "INSERT INTO MumbleLinks (Name, Url, DisplayOrder, IsSelected, Hotkey, ServerGroupId, FolderId)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6)",
            params![
                name.trim(),
                url.trim(),
                display_order,
                hotkey,
                server_group_id,
                folder_id
            ],
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
        server_group_id: i64,
        folder_id: Option<i64>,
    ) -> Result<(), String> {
        validate_link_name(&name)?;
        validate_link_url(&url)?;
        let conn = self.connection()?;
        self.validate_mumble_link_placement(&conn, server_group_id, folder_id)?;
        let n = conn
            .execute(
                "UPDATE MumbleLinks
             SET Name = ?1, Url = ?2, DisplayOrder = ?3, Hotkey = ?4,
                 ServerGroupId = ?5, FolderId = ?6
             WHERE Id = ?7",
                params![
                    name.trim(),
                    url.trim(),
                    display_order,
                    hotkey,
                    server_group_id,
                    folder_id,
                    link_id
                ],
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Mumble link not found".to_string());
        }
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

    pub fn create_mumble_server_group(
        &self,
        name: String,
        display_order: i64,
    ) -> Result<(), String> {
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
        conn.execute(
            "DELETE FROM MumbleLinks WHERE ServerGroupId = ?1",
            params![group_id],
        )
        .map_err(|e| e.to_string())?;
        self.delete_mumble_folders_for_server(&conn, group_id)?;
        conn.execute("DELETE FROM MumbleServerGroups WHERE Id = ?1", params![group_id])
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
            s.and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(default_value)
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
        self.set_app_setting(
            "DrawerScreenIndex".to_string(),
            settings.screen_index.to_string(),
        )?;
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
        conn.query_row("SELECT Value FROM AppSettings WHERE Key = ?1", [key], |r| {
            r.get(0)
        })
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

    fn load_grid_layout_prefs_store(&self) -> Result<GridLayoutPrefsStore, String> {
        let raw = self.get_app_setting(GRID_LAYOUT_PREFS_KEY.to_string())?;
        match raw.filter(|s| !s.trim().is_empty()) {
            None => Ok(GridLayoutPrefsStore::default()),
            Some(s) => serde_json::from_str(&s).map_err(|e| e.to_string()),
        }
    }

    pub fn get_grid_layout_prefs(&self, profile_id: i64) -> Result<Option<GridLayoutFormPrefs>, String> {
        let store = self.load_grid_layout_prefs_store()?;
        let key = profile_id.to_string();
        let Some(mut prefs) = store.by_profile.get(&key).cloned() else {
            return Ok(None);
        };
        prefs.normalize();
        Ok(Some(prefs))
    }

    pub fn set_grid_layout_prefs(&self, profile_id: i64, mut prefs: GridLayoutFormPrefs) -> Result<(), String> {
        prefs.normalize();
        let mut store = self.load_grid_layout_prefs_store()?;
        store.version = 1;
        store
            .by_profile
            .insert(profile_id.to_string(), prefs);
        let json = serde_json::to_string(&store).map_err(|e| e.to_string())?;
        if json.len() > 512 * 1024 {
            return Err("Grid layout preferences are too large to store.".to_string());
        }
        self.set_app_setting(GRID_LAYOUT_PREFS_KEY.to_string(), json)
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
    let allowed = value.starts_with("mumble://") || value.starts_with("https://");
    if !allowed {
        return Err("Link URL must start with mumble:// or https://".to_string());
    }
    Ok(())
}
