use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub deleted_at: Option<String>,
    pub is_active: bool,
    pub switch_hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshot {
    pub app: &'static str,
    pub backend_ready: bool,
    pub active_profile_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailConfig {
    pub width: i64,
    pub height: i64,
    pub x: i64,
    pub y: i64,
    pub opacity: f64,
    pub focus_border_color: String,
    pub focus_border_thickness: i64,
    pub show_title_overlay: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailSetting {
    pub window_title: String,
    pub config: ThumbnailConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientGroup {
    pub id: i64,
    pub profile_id: i64,
    pub name: String,
    pub display_order: i64,
    pub cycle_forward_hotkey: String,
    pub cycle_backward_hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientGroupMember {
    pub window_title: String,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientGroupDetail {
    pub id: i64,
    pub profile_id: i64,
    pub name: String,
    pub display_order: i64,
    pub cycle_forward_hotkey: String,
    pub cycle_backward_hotkey: String,
    pub members: Vec<ClientGroupMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MumbleServerGroup {
    pub id: i64,
    pub name: String,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MumbleLink {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub display_order: i64,
    pub is_selected: bool,
    pub hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MumbleLinksOverlaySettings {
    pub always_on_top: bool,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawerSettings {
    pub screen_index: i64,
    pub hardware_id: String,
    pub side: String,
    pub width: i64,
    pub height: i64,
    pub is_visible: bool,
    pub is_enabled: bool,
    pub selected_mumble_server_group_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfoDto {
    pub index: i32,
    pub name: String,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub work_left: i32,
    pub work_top: i32,
    pub work_right: i32,
    pub work_bottom: i32,
    pub is_primary: bool,
    pub hardware_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridLayoutPayload {
    pub profile_id: i64,
    pub grid_cell_width: i64,
    pub grid_cell_height: Option<i64>,
    pub grid_cell_ratio: Option<String>,
    pub grid_start_x: i64,
    pub grid_start_y: i64,
    pub grid_columns: i64,
    pub selected_group_id: Option<i64>,
    pub only_affect_active_thumbnails: bool,
    #[serde(default)]
    pub selected_monitor_index: Option<i64>,
    /// When set, that thumbnail is placed first in the grid and its saved position
    /// defines the grid origin (after subtracting the selected monitor work-area offset).
    #[serde(default)]
    pub grid_anchor_window_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridLayoutPreviewItem {
    pub window_title: String,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}
