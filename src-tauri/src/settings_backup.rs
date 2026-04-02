//! Full YAEP SQLite user settings export / import (profiles, thumbnails, mumble, widgets, app keys).

use std::collections::HashSet;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::DbService;

const FORMAT_VERSION: u32 = 1;
const MAX_IMPORT_BYTES: usize = 40 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YaepSettingsBundle {
    pub format_version: u32,
    #[serde(default)]
    pub exported_at: String,
    pub app_settings: Vec<AppKv>,
    pub profiles: Vec<ProfileRow>,
    pub processes_to_preview: Vec<ProcessPreviewRow>,
    pub thumbnail_default_config: Vec<ThumbnailDefaultRow>,
    pub thumbnail_settings: Vec<ThumbnailSettingsRow>,
    pub eve_log_settings: Vec<EveLogRow>,
    pub eve_chat_channels: Vec<EveChatChannelRow>,
    pub client_groups: Vec<ClientGroupRow>,
    pub client_group_members: Vec<ClientGroupMemberRow>,
    pub mumble_server_groups: Vec<MumbleServerGroupRow>,
    pub mumble_folders: Vec<MumbleFolderRow>,
    pub mumble_links: Vec<MumbleLinkRow>,
    pub mumble_link_groups: Vec<MumbleLinkGroupRow>,
    pub mumble_links_overlay_settings: Vec<MumbleOverlayRow>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppKv {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRow {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
    pub is_active: bool,
    pub switch_hotkey: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessPreviewRow {
    pub profile_id: i64,
    pub process_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailDefaultRow {
    pub profile_id: i64,
    pub width: i64,
    pub height: i64,
    pub x: i64,
    pub y: i64,
    pub opacity: f64,
    pub focus_border_color: String,
    pub focus_border_thickness: i64,
    pub decloak_flash_color: String,
    pub decloak_flash_thickness: i64,
    pub decloak_flash_duration_ms: i64,
    pub show_title_overlay: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailSettingsRow {
    pub profile_id: i64,
    pub window_title: String,
    pub width: i64,
    pub height: i64,
    pub x: i64,
    pub y: i64,
    pub opacity: f64,
    pub focus_border_color: String,
    pub focus_border_thickness: i64,
    pub decloak_flash_color: String,
    pub decloak_flash_thickness: i64,
    pub decloak_flash_duration_ms: i64,
    pub show_title_overlay: bool,
    #[serde(default)]
    pub character_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EveLogRow {
    pub profile_id: i64,
    pub chat_logs_path: String,
    pub game_logs_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EveChatChannelRow {
    pub id: i64,
    pub profile_id: i64,
    pub channel_type: String,
    pub channel_name: String,
    pub background_color: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientGroupRow {
    pub id: i64,
    pub profile_id: i64,
    pub name: String,
    pub display_order: i64,
    pub cycle_forward_hotkey: String,
    pub cycle_backward_hotkey: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientGroupMemberRow {
    pub group_id: i64,
    pub window_title: String,
    pub display_order: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MumbleServerGroupRow {
    pub id: i64,
    pub name: String,
    pub display_order: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MumbleFolderRow {
    pub id: i64,
    pub server_group_id: i64,
    #[serde(default)]
    pub parent_folder_id: Option<i64>,
    pub name: String,
    pub display_order: i64,
    #[serde(default)]
    pub icon_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MumbleLinkRow {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub display_order: i64,
    pub is_selected: bool,
    pub hotkey: String,
    pub server_group_id: i64,
    #[serde(default)]
    pub folder_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MumbleLinkGroupRow {
    pub link_id: i64,
    pub group_id: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MumbleOverlayRow {
    pub id: i64,
    pub always_on_top: bool,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

pub fn export_bundle(db: &DbService) -> Result<String, String> {
    let conn = db.db_conn()?;
    let exported_at = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );

    let mut stmt = conn
        .prepare("SELECT Key, Value FROM AppSettings ORDER BY Key ASC")
        .map_err(|e| e.to_string())?;
    let app_settings: Vec<AppKv> = stmt
        .query_map([], |row| {
            Ok(AppKv {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT Id, Name, DeletedAt, IsActive, SwitchHotkey FROM Profile ORDER BY Id ASC",
        )
        .map_err(|e| e.to_string())?;
    let profiles: Vec<ProfileRow> = stmt
        .query_map([], |row| {
            Ok(ProfileRow {
                id: row.get(0)?,
                name: row.get(1)?,
                deleted_at: row.get(2)?,
                is_active: row.get::<_, i64>(3)? == 1,
                switch_hotkey: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT ProfileId, ProcessName FROM ProcessesToPreview ORDER BY ProfileId, ProcessName")
        .map_err(|e| e.to_string())?;
    let processes_to_preview: Vec<ProcessPreviewRow> = stmt
        .query_map([], |row| {
            Ok(ProcessPreviewRow {
                profile_id: row.get(0)?,
                process_name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness,
                    DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay
             FROM ThumbnailDefaultConfig ORDER BY ProfileId",
        )
        .map_err(|e| e.to_string())?;
    let thumbnail_default_config: Vec<ThumbnailDefaultRow> = stmt
        .query_map([], |row| {
            Ok(ThumbnailDefaultRow {
                profile_id: row.get(0)?,
                width: row.get(1)?,
                height: row.get(2)?,
                x: row.get(3)?,
                y: row.get(4)?,
                opacity: row.get(5)?,
                focus_border_color: row.get(6)?,
                focus_border_thickness: row.get(7)?,
                decloak_flash_color: row.get(8)?,
                decloak_flash_thickness: row.get(9)?,
                decloak_flash_duration_ms: row.get(10)?,
                show_title_overlay: row.get::<_, i64>(11)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT ProfileId, WindowTitle, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness,
                    DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay, CharacterId
             FROM ThumbnailSettings ORDER BY ProfileId, WindowTitle",
        )
        .map_err(|e| e.to_string())?;
    let thumbnail_settings: Vec<ThumbnailSettingsRow> = stmt
        .query_map([], |row| {
            Ok(ThumbnailSettingsRow {
                profile_id: row.get(0)?,
                window_title: row.get(1)?,
                width: row.get(2)?,
                height: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
                opacity: row.get(6)?,
                focus_border_color: row.get(7)?,
                focus_border_thickness: row.get(8)?,
                decloak_flash_color: row.get(9)?,
                decloak_flash_thickness: row.get(10)?,
                decloak_flash_duration_ms: row.get(11)?,
                show_title_overlay: row.get::<_, i64>(12)? == 1,
                character_id: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT ProfileId, ChatLogsPath, GameLogsPath FROM EveLogSettings ORDER BY ProfileId")
        .map_err(|e| e.to_string())?;
    let eve_log_settings: Vec<EveLogRow> = stmt
        .query_map([], |row| {
            Ok(EveLogRow {
                profile_id: row.get(0)?,
                chat_logs_path: row.get(1)?,
                game_logs_path: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT Id, ProfileId, ChannelType, ChannelName, BackgroundColor FROM EveChatChannels ORDER BY Id",
        )
        .map_err(|e| e.to_string())?;
    let eve_chat_channels: Vec<EveChatChannelRow> = stmt
        .query_map([], |row| {
            Ok(EveChatChannelRow {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                channel_type: row.get(2)?,
                channel_name: row.get(3)?,
                background_color: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT Id, ProfileId, Name, DisplayOrder, CycleForwardHotkey, CycleBackwardHotkey
             FROM ClientGroups ORDER BY ProfileId, DisplayOrder, Id",
        )
        .map_err(|e| e.to_string())?;
    let client_groups: Vec<ClientGroupRow> = stmt
        .query_map([], |row| {
            Ok(ClientGroupRow {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                name: row.get(2)?,
                display_order: row.get(3)?,
                cycle_forward_hotkey: row.get(4)?,
                cycle_backward_hotkey: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT GroupId, WindowTitle, DisplayOrder FROM ClientGroupMembers ORDER BY GroupId, DisplayOrder",
        )
        .map_err(|e| e.to_string())?;
    let client_group_members: Vec<ClientGroupMemberRow> = stmt
        .query_map([], |row| {
            Ok(ClientGroupMemberRow {
                group_id: row.get(0)?,
                window_title: row.get(1)?,
                display_order: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT Id, Name, DisplayOrder FROM MumbleServerGroups ORDER BY DisplayOrder, Id")
        .map_err(|e| e.to_string())?;
    let mumble_server_groups: Vec<MumbleServerGroupRow> = stmt
        .query_map([], |row| {
            Ok(MumbleServerGroupRow {
                id: row.get(0)?,
                name: row.get(1)?,
                display_order: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT Id, ServerGroupId, ParentFolderId, Name, DisplayOrder, IconKey FROM MumbleFolders ORDER BY Id",
        )
        .map_err(|e| e.to_string())?;
    let mumble_folders: Vec<MumbleFolderRow> = stmt
        .query_map([], |row| {
            Ok(MumbleFolderRow {
                id: row.get(0)?,
                server_group_id: row.get(1)?,
                parent_folder_id: row.get(2)?,
                name: row.get(3)?,
                display_order: row.get(4)?,
                icon_key: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT Id, Name, Url, DisplayOrder, IsSelected, Hotkey, ServerGroupId, FolderId FROM MumbleLinks ORDER BY Id",
        )
        .map_err(|e| e.to_string())?;
    let mumble_links: Vec<MumbleLinkRow> = stmt
        .query_map([], |row| {
            Ok(MumbleLinkRow {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                display_order: row.get(3)?,
                is_selected: row.get::<_, i64>(4)? == 1,
                hotkey: row.get(5)?,
                server_group_id: row.get(6)?,
                folder_id: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT LinkId, GroupId FROM MumbleLinkGroups ORDER BY LinkId, GroupId")
        .map_err(|e| e.to_string())?;
    let mumble_link_groups: Vec<MumbleLinkGroupRow> = stmt
        .query_map([], |row| {
            Ok(MumbleLinkGroupRow {
                link_id: row.get(0)?,
                group_id: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT Id, AlwaysOnTop, X, Y, Width, Height FROM MumbleLinksOverlaySettings ORDER BY Id")
        .map_err(|e| e.to_string())?;
    let mumble_links_overlay_settings: Vec<MumbleOverlayRow> = stmt
        .query_map([], |row| {
            Ok(MumbleOverlayRow {
                id: row.get(0)?,
                always_on_top: row.get::<_, i64>(1)? == 1,
                x: row.get(2)?,
                y: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let bundle = YaepSettingsBundle {
        format_version: FORMAT_VERSION,
        exported_at,
        app_settings,
        profiles,
        processes_to_preview,
        thumbnail_default_config,
        thumbnail_settings,
        eve_log_settings,
        eve_chat_channels,
        client_groups,
        client_group_members,
        mumble_server_groups,
        mumble_folders,
        mumble_links,
        mumble_link_groups,
        mumble_links_overlay_settings,
    };

    serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())
}

pub fn import_bundle(db: &DbService, json: &str) -> Result<(), String> {
    if json.len() > MAX_IMPORT_BYTES {
        return Err("Backup file is too large.".to_string());
    }
    let mut bundle: YaepSettingsBundle =
        serde_json::from_str(json).map_err(|e| format!("Invalid backup file: {e}"))?;
    if bundle.format_version != FORMAT_VERSION {
        return Err(
            "This backup was created by a different YAEP version and cannot be imported here."
                .to_string(),
        );
    }
    validate_bundle(&bundle)?;
    normalize_active_profile(&mut bundle.profiles);

    let mut conn = db.db_conn()?;
    conn.execute("PRAGMA foreign_keys = OFF", [])
        .map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM ClientGroupMembers", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM ClientGroups", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM EveChatChannels", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM EveLogSettings", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM ThumbnailSettings", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM ThumbnailDefaultConfig", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM ProcessesToPreview", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM Profile", [])
        .map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM MumbleLinkGroups", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM MumbleLinks", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM MumbleFolders", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM MumbleServerGroups", [])
        .map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM MumbleLinksOverlaySettings", [])
        .map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM AppSettings", [])
        .map_err(|e| e.to_string())?;

    for p in &bundle.profiles {
        tx.execute(
            "INSERT INTO Profile (Id, Name, DeletedAt, IsActive, SwitchHotkey)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                p.id,
                p.name,
                p.deleted_at,
                if p.is_active { 1 } else { 0 },
                p.switch_hotkey
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.processes_to_preview {
        tx.execute(
            "INSERT INTO ProcessesToPreview (ProfileId, ProcessName) VALUES (?1, ?2)",
            params![row.profile_id, row.process_name],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.thumbnail_default_config {
        tx.execute(
            "INSERT INTO ThumbnailDefaultConfig
             (ProfileId, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness,
              DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                row.profile_id,
                row.width,
                row.height,
                row.x,
                row.y,
                row.opacity,
                row.focus_border_color,
                row.focus_border_thickness,
                row.decloak_flash_color,
                row.decloak_flash_thickness,
                row.decloak_flash_duration_ms,
                if row.show_title_overlay { 1 } else { 0 }
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.thumbnail_settings {
        tx.execute(
            "INSERT INTO ThumbnailSettings
             (ProfileId, WindowTitle, Width, Height, X, Y, Opacity, FocusBorderColor, FocusBorderThickness,
              DecloakFlashColor, DecloakFlashThickness, DecloakFlashDurationMs, ShowTitleOverlay, CharacterId)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                row.profile_id,
                row.window_title,
                row.width,
                row.height,
                row.x,
                row.y,
                row.opacity,
                row.focus_border_color,
                row.focus_border_thickness,
                row.decloak_flash_color,
                row.decloak_flash_thickness,
                row.decloak_flash_duration_ms,
                if row.show_title_overlay { 1 } else { 0 },
                row.character_id
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.eve_log_settings {
        tx.execute(
            "INSERT INTO EveLogSettings (ProfileId, ChatLogsPath, GameLogsPath) VALUES (?1, ?2, ?3)",
            params![row.profile_id, row.chat_logs_path, row.game_logs_path],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.eve_chat_channels {
        tx.execute(
            "INSERT INTO EveChatChannels (Id, ProfileId, ChannelType, ChannelName, BackgroundColor)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                row.id,
                row.profile_id,
                row.channel_type,
                row.channel_name,
                row.background_color
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.client_groups {
        tx.execute(
            "INSERT INTO ClientGroups (Id, ProfileId, Name, DisplayOrder, CycleForwardHotkey, CycleBackwardHotkey)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                row.id,
                row.profile_id,
                row.name,
                row.display_order,
                row.cycle_forward_hotkey,
                row.cycle_backward_hotkey
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.client_group_members {
        tx.execute(
            "INSERT INTO ClientGroupMembers (GroupId, WindowTitle, DisplayOrder) VALUES (?1, ?2, ?3)",
            params![row.group_id, row.window_title, row.display_order],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.mumble_server_groups {
        tx.execute(
            "INSERT INTO MumbleServerGroups (Id, Name, DisplayOrder) VALUES (?1, ?2, ?3)",
            params![row.id, row.name, row.display_order],
        )
        .map_err(|e| e.to_string())?;
    }

    let folder_order = ordered_mumble_folders(&bundle.mumble_folders)?;
    for row in folder_order {
        tx.execute(
            "INSERT INTO MumbleFolders (Id, ServerGroupId, ParentFolderId, Name, DisplayOrder, IconKey)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                row.id,
                row.server_group_id,
                row.parent_folder_id,
                row.name,
                row.display_order,
                row.icon_key
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.mumble_links {
        tx.execute(
            "INSERT INTO MumbleLinks (Id, Name, Url, DisplayOrder, IsSelected, Hotkey, ServerGroupId, FolderId)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                row.id,
                row.name,
                row.url,
                row.display_order,
                if row.is_selected { 1 } else { 0 },
                row.hotkey,
                row.server_group_id,
                row.folder_id
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    for row in &bundle.mumble_link_groups {
        tx.execute(
            "INSERT INTO MumbleLinkGroups (LinkId, GroupId) VALUES (?1, ?2)",
            params![row.link_id, row.group_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if bundle.mumble_links_overlay_settings.is_empty() {
        tx.execute(
            "INSERT INTO MumbleLinksOverlaySettings (AlwaysOnTop, X, Y, Width, Height) VALUES (1, 100, 100, 300, 400)",
            [],
        )
        .map_err(|e| e.to_string())?;
    } else {
        for row in &bundle.mumble_links_overlay_settings {
            tx.execute(
                "INSERT INTO MumbleLinksOverlaySettings (Id, AlwaysOnTop, X, Y, Width, Height)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    row.id,
                    if row.always_on_top { 1 } else { 0 },
                    row.x,
                    row.y,
                    row.width,
                    row.height
                ],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    for kv in &bundle.app_settings {
        tx.execute(
            "INSERT INTO AppSettings (Key, Value) VALUES (?1, ?2)",
            params![kv.key, kv.value],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    sync_autoincrement_seq(&conn, "Profile", "Id")?;
    sync_autoincrement_seq(&conn, "MumbleServerGroups", "Id")?;
    sync_autoincrement_seq(&conn, "MumbleFolders", "Id")?;
    sync_autoincrement_seq(&conn, "MumbleLinks", "Id")?;
    sync_autoincrement_seq(&conn, "ClientGroups", "Id")?;
    sync_autoincrement_seq(&conn, "EveChatChannels", "Id")?;
    sync_autoincrement_seq(&conn, "MumbleLinksOverlaySettings", "Id")?;

    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| e.to_string())?;

    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if integrity.to_ascii_lowercase() != "ok" {
        return Err("Database integrity check failed after import.".to_string());
    }

    Ok(())
}

fn validate_bundle(bundle: &YaepSettingsBundle) -> Result<(), String> {
    if bundle.profiles.is_empty() {
        return Err("Backup contains no profiles.".to_string());
    }

    let profile_ids: HashSet<i64> = bundle.profiles.iter().map(|p| p.id).collect();
    if profile_ids.len() != bundle.profiles.len() {
        return Err("Backup has duplicate profile ids.".to_string());
    }
    let mut seen_names = HashSet::<String>::new();
    for p in &bundle.profiles {
        let key = p.name.trim().to_string();
        if key.is_empty() {
            return Err("Backup contains a profile with an empty name.".to_string());
        }
        if !seen_names.insert(key) {
            return Err("Backup has duplicate profile names.".to_string());
        }
    }

    let group_ids: HashSet<i64> = bundle.client_groups.iter().map(|g| g.id).collect();
    if group_ids.len() != bundle.client_groups.len() {
        return Err("Backup has duplicate client group ids.".to_string());
    }
    for g in &bundle.client_groups {
        if !profile_ids.contains(&g.profile_id) {
            return Err("Backup references a client group for an unknown profile.".to_string());
        }
    }
    for m in &bundle.client_group_members {
        if !group_ids.contains(&m.group_id) {
            return Err("Backup has a client group member for an unknown group.".to_string());
        }
    }

    let mumble_gids: HashSet<i64> = bundle.mumble_server_groups.iter().map(|g| g.id).collect();
    if mumble_gids.len() != bundle.mumble_server_groups.len() {
        return Err("Backup has duplicate Mumble server group ids.".to_string());
    }
    if mumble_gids.is_empty() {
        return Err("Backup contains no Mumble server groups.".to_string());
    }

    let folder_ids: HashSet<i64> = bundle.mumble_folders.iter().map(|f| f.id).collect();
    if folder_ids.len() != bundle.mumble_folders.len() {
        return Err("Backup has duplicate Mumble folder ids.".to_string());
    }
    for f in &bundle.mumble_folders {
        if !mumble_gids.contains(&f.server_group_id) {
            return Err("Backup has a Mumble folder under an unknown server group.".to_string());
        }
        if let Some(p) = f.parent_folder_id {
            if !folder_ids.contains(&p) {
                return Err("Backup has a Mumble folder with an invalid parent folder.".to_string());
            }
        }
    }

    let link_ids: HashSet<i64> = bundle.mumble_links.iter().map(|l| l.id).collect();
    if link_ids.len() != bundle.mumble_links.len() {
        return Err("Backup has duplicate Mumble link ids.".to_string());
    }
    for link in &bundle.mumble_links {
        if !mumble_gids.contains(&link.server_group_id) {
            return Err("Backup has a Mumble link in an unknown server group.".to_string());
        }
        if let Some(fid) = link.folder_id {
            if !folder_ids.contains(&fid) {
                return Err("Backup has a Mumble link in an unknown folder.".to_string());
            }
        }
    }

    for lg in &bundle.mumble_link_groups {
        if !link_ids.contains(&lg.link_id) {
            return Err("Backup has an invalid Mumble link group row.".to_string());
        }
        if !mumble_gids.contains(&lg.group_id) {
            return Err("Backup has an invalid Mumble link group row.".to_string());
        }
    }

    macro_rules! check_pid {
        ($rows:expr, $label:literal) => {
            for row in $rows {
                let pid = row.profile_id;
                if !profile_ids.contains(&pid) {
                    return Err(concat!("Backup references an unknown profile in ", $label, ".").to_string());
                }
            }
        };
    }

    check_pid!(&bundle.processes_to_preview, "process list");
    check_pid!(&bundle.thumbnail_default_config, "thumbnail defaults");
    for t in &bundle.thumbnail_settings {
        if !profile_ids.contains(&t.profile_id) {
            return Err("Backup references an unknown profile in thumbnail settings.".to_string());
        }
    }
    check_pid!(&bundle.eve_log_settings, "EVE log settings");
    let channel_ids: HashSet<i64> = bundle.eve_chat_channels.iter().map(|c| c.id).collect();
    if channel_ids.len() != bundle.eve_chat_channels.len() {
        return Err("Backup has duplicate EVE chat channel ids.".to_string());
    }
    for ch in &bundle.eve_chat_channels {
        if !profile_ids.contains(&ch.profile_id) {
            return Err("Backup references an unknown profile in EVE chat channels.".to_string());
        }
    }

    if bundle.mumble_links_overlay_settings.len() > 1 {
        let oids: HashSet<i64> = bundle
            .mumble_links_overlay_settings
            .iter()
            .map(|r| r.id)
            .collect();
        if oids.len() != bundle.mumble_links_overlay_settings.len() {
            return Err("Backup has duplicate Mumble overlay settings ids.".to_string());
        }
    }

    for kv in &bundle.app_settings {
        if kv.key.trim().is_empty() {
            return Err("Backup contains an empty app settings key.".to_string());
        }
        if kv.value.len() > 512 * 1024 {
            return Err("Backup contains an app setting value that is too large.".to_string());
        }
    }

    Ok(())
}

fn normalize_active_profile(profiles: &mut Vec<ProfileRow>) {
    if profiles.is_empty() {
        return;
    }
    let active_ids: Vec<i64> = profiles.iter().filter(|p| p.is_active).map(|p| p.id).collect();
    let chosen = if active_ids.len() == 1 {
        active_ids[0]
    } else if active_ids.is_empty() {
        profiles.iter().map(|p| p.id).min().unwrap_or(0)
    } else {
        *active_ids.iter().min().unwrap_or(&active_ids[0])
    };
    for p in profiles.iter_mut() {
        p.is_active = p.id == chosen;
    }
}

fn ordered_mumble_folders(folders: &[MumbleFolderRow]) -> Result<Vec<&MumbleFolderRow>, String> {
    let id_set: HashSet<i64> = folders.iter().map(|f| f.id).collect();
    for f in folders {
        if let Some(p) = f.parent_folder_id {
            if !id_set.contains(&p) {
                return Err("Invalid Mumble folder parent reference in backup.".to_string());
            }
        }
    }

    let mut remaining: Vec<&MumbleFolderRow> = folders.iter().collect();
    let mut ordered: Vec<&MumbleFolderRow> = Vec::new();
    while !remaining.is_empty() {
        let before = remaining.len();
        let inserted: HashSet<i64> = ordered.iter().map(|f| f.id).collect();
        let mut next = Vec::new();
        for f in remaining {
            let ready = match f.parent_folder_id {
                None => true,
                Some(p) => inserted.contains(&p),
            };
            if ready {
                ordered.push(f);
            } else {
                next.push(f);
            }
        }
        remaining = next;
        if remaining.len() == before {
            return Err("Mumble folder hierarchy in backup has a cycle.".to_string());
        }
    }
    Ok(ordered)
}

fn sync_autoincrement_seq(conn: &rusqlite::Connection, table: &str, id_col: &str) -> Result<(), String> {
    let max: i64 = conn
        .query_row(
            &format!("SELECT IFNULL(MAX({id_col}), 0) FROM {table}"),
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM sqlite_sequence WHERE name = ?1", [table])
        .map_err(|e| e.to_string())?;
    if max > 0 {
        conn.execute(
            "INSERT INTO sqlite_sequence (name, seq) VALUES (?1, ?2)",
            params![table, max],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
