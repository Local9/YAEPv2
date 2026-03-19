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
