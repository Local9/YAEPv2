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
    pub decloak_flash_color: String,
    pub decloak_flash_thickness: i64,
    pub decloak_flash_duration_ms: i64,
    pub show_title_overlay: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailSetting {
    pub window_title: String,
    pub config: ThumbnailConfig,
    #[serde(default)]
    pub character_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveLogSettings {
    pub chat_logs_path: String,
    pub game_logs_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveFolderSettings {
    pub chat_logs_path: String,
    pub game_logs_path: String,
    pub pi_templates_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiTemplateLink {
    /// Route path (ordered pin/node references).
    #[serde(rename = "P")]
    pub path: Vec<i64>,
    /// Routed quantity.
    #[serde(rename = "Q")]
    pub quantity: i64,
    /// Routed type ID.
    #[serde(rename = "T")]
    pub type_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiTemplatePin {
    /// Height / layer hint for placement.
    #[serde(rename = "H")]
    pub head: i64,
    /// Latitude (radians).
    #[serde(rename = "La")]
    pub latitude: f64,
    /// Longitude (radians).
    #[serde(rename = "Lo")]
    pub longitude: f64,
    /// Structure/schematic ID (nullable in template exports).
    #[serde(rename = "S")]
    pub schematic_id: Option<i64>,
    /// Pin type ID.
    #[serde(rename = "T")]
    pub type_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiTemplateLegendItem {
    /// Direction/group marker used by the template format.
    #[serde(rename = "D")]
    pub icon_id: i64,
    /// Link/legend level.
    #[serde(rename = "Lv")]
    pub level: i64,
    /// Structure/type reference ID.
    #[serde(rename = "S")]
    pub type_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiTemplate {
    /// Command Center Upgrades skill level.
    #[serde(rename = "CmdCtrLv")]
    pub command_center_level: i64,
    /// User comment/label for the template.
    #[serde(rename = "Cmt")]
    pub comment: String,
    /// Planet diameter.
    #[serde(rename = "Diam")]
    pub diameter: f64,
    /// Link/legend definitions (`D`, `Lv`, `S`).
    #[serde(rename = "L")]
    pub legend: Vec<PiTemplateLegendItem>,
    /// Structure placements (`H`, `La`, `Lo`, `S`, `T`).
    #[serde(rename = "P")]
    pub pins: Vec<PiTemplatePin>,
    /// Planet type identifier from the template.
    #[serde(rename = "Pln")]
    pub planet_type_id: i64,
    /// Route definitions (`P`, `Q`, `T`).
    #[serde(rename = "R")]
    pub routes: Vec<PiTemplateLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PiTemplateValidationIssue {
    pub file_name: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveChatChannel {
    pub id: i64,
    pub profile_id: i64,
    pub channel_type: String,
    pub channel_name: String,
    pub background_color: String,
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
pub struct MumbleFolder {
    pub id: i64,
    pub server_group_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<i64>,
    pub name: String,
    pub display_order: i64,
    /// Lucide-style slug (e.g. `headphones`); omitted or null means default icon in the UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_key: Option<String>,
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
    pub server_group_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<i64>,
}

/// Flat snapshot for building the server / folder / link tree in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MumbleTreeSnapshot {
    pub groups: Vec<MumbleServerGroup>,
    pub folders: Vec<MumbleFolder>,
    pub links: Vec<MumbleLink>,
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

fn default_widget_overlay_visible() -> bool {
    true
}

fn default_widget_overlay_show_browser() -> bool {
    true
}

fn default_widget_overlay_show_fleet_motd() -> bool {
    true
}

fn default_widget_overlay_show_intel_feed() -> bool {
    true
}

fn default_widget_overlay_show_mumble_links() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserQuickLink {
    pub id: String,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetOverlaySettings {
    pub enabled: bool,
    /// When `enabled` is true, controls whether the overlay window is actually shown (can be toggled from tray).
    #[serde(default = "default_widget_overlay_visible")]
    pub visible: bool,
    pub monitor_index: i64,
    #[serde(default = "default_widget_overlay_show_browser")]
    pub show_browser_widget: bool,
    #[serde(default = "default_widget_overlay_show_fleet_motd")]
    pub show_fleet_motd_widget: bool,
    #[serde(default = "default_widget_overlay_show_intel_feed")]
    pub show_intel_feed_widget: bool,
    #[serde(default = "default_widget_overlay_show_mumble_links")]
    pub show_mumble_links_widget: bool,
    /// When true (after hotkey/tray), non-pinned widgets are hidden; see `browser_always_displayed`.
    #[serde(default)]
    pub widgets_suppressed: bool,
    /// Browser widget stays visible while `widgets_suppressed` is true.
    #[serde(default)]
    pub browser_always_displayed: bool,
    /// Fleet MOTD widget stays visible while `widgets_suppressed` is true.
    #[serde(default)]
    pub fleet_motd_always_displayed: bool,
    /// Intel feed widget stays visible while `widgets_suppressed` is true.
    #[serde(default)]
    pub intel_feed_always_displayed: bool,
    /// Mumble links widget stays visible while `widgets_suppressed` is true.
    #[serde(default)]
    pub mumble_links_always_displayed: bool,
    /// Global hotkey string (e.g. `Ctrl+Shift+W`) to toggle `widgets_suppressed`.
    #[serde(default)]
    pub toggle_hotkey: String,
    #[serde(default = "default_browser_quick_links_vec")]
    pub browser_quick_links: Vec<BrowserQuickLink>,
    #[serde(default)]
    pub browser_default_url: Option<String>,
    pub layout: WidgetOverlayLayout,
}

pub fn default_browser_quick_links() -> Vec<BrowserQuickLink> {
    vec![
        BrowserQuickLink {
            id: "eve-uni-wiki".into(),
            url: "https://wiki.eveuniversity.org/".into(),
            title: "EVE University".into(),
        },
        BrowserQuickLink {
            id: "dotlan".into(),
            url: "https://evemaps.dotlan.net/".into(),
            title: "dotlan".into(),
        },
        BrowserQuickLink {
            id: "janice".into(),
            url: "https://janice.e-351.com/".into(),
            title: "Janice".into(),
        },
    ]
}

fn default_browser_quick_links_vec() -> Vec<BrowserQuickLink> {
    default_browser_quick_links()
}

fn default_mumble_links_widget_frame() -> WidgetFrame {
    WidgetFrame {
        x: 24.0,
        y: 520.0,
        width: 200.0,
        height: 44.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetOverlayLayout {
    #[serde(default)]
    pub browser: WidgetBrowserFrame,
    #[serde(default)]
    pub fleet_motd: WidgetFrame,
    #[serde(default)]
    pub intel_feed: WidgetFrame,
    #[serde(default = "default_mumble_links_widget_frame")]
    pub mumble_links: WidgetFrame,
}

impl Default for WidgetOverlayLayout {
    fn default() -> Self {
        Self {
            browser: WidgetBrowserFrame::default(),
            fleet_motd: WidgetFrame::default(),
            intel_feed: WidgetFrame::default(),
            mumble_links: default_mumble_links_widget_frame(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for WidgetFrame {
    fn default() -> Self {
        Self {
            x: 24.0,
            y: 24.0,
            width: 420.0,
            height: 180.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetBrowserFrame {
    #[serde(default = "default_browser_widget_url")]
    pub url: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn default_browser_widget_url() -> String {
    String::new()
}

impl Default for WidgetBrowserFrame {
    fn default() -> Self {
        Self {
            url: default_browser_widget_url(),
            x: 400.0,
            y: 48.0,
            width: 480.0,
            height: 360.0,
        }
    }
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

/// Grid Layout page form state persisted per profile (SQLite `AppSettings` / export bundle).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridLayoutFormPrefs {
    pub aspect_ratio: String,
    pub grid_cell_width: i64,
    pub grid_cell_height: i64,
    pub grid_start_x: i64,
    pub grid_start_y: i64,
    pub grid_columns: i64,
    pub only_affect_active_thumbnails: bool,
    #[serde(default)]
    pub selected_monitor_index: String,
    #[serde(default)]
    pub selected_anchor_title: String,
}

impl GridLayoutFormPrefs {
    pub fn normalize(&mut self) {
        self.aspect_ratio = Self::sanitize_aspect_ratio(&self.aspect_ratio);
        self.grid_cell_width = Self::clamp_i64(self.grid_cell_width, 192, 960);
        self.grid_cell_height = Self::clamp_i64(self.grid_cell_height, 108, 540);
        self.grid_start_x = Self::clamp_i64(self.grid_start_x, -10_000, 31_000);
        self.grid_start_y = Self::clamp_i64(self.grid_start_y, -10_000, 31_000);
        self.grid_columns = Self::clamp_i64(self.grid_columns, 1, 10);
        self.selected_monitor_index = self.selected_monitor_index.trim().to_string();
        if !self.selected_monitor_index.is_empty()
            && self.selected_monitor_index.parse::<i32>().is_err()
        {
            self.selected_monitor_index.clear();
        }
        let anchor = self.selected_anchor_title.trim().chars().take(2048).collect::<String>();
        self.selected_anchor_title = anchor;
    }

    fn clamp_i64(n: i64, lo: i64, hi: i64) -> i64 {
        n.max(lo).min(hi)
    }

    fn sanitize_aspect_ratio(raw: &str) -> String {
        let t = raw.trim();
        let parts: Vec<&str> = t.split(':').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if parts.len() != 2 {
            return "16:9".to_string();
        }
        let a: f64 = parts[0].parse().unwrap_or(0.0);
        let b: f64 = parts[1].parse().unwrap_or(0.0);
        if !a.is_finite() || !b.is_finite() || a <= 0.0 || b <= 0.0 {
            return "16:9".to_string();
        }
        format!("{}:{}", parts[0], parts[1])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveDetectedProfile {
    pub server_name: String,
    pub profile_name: String,
    pub full_path: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveProfileCharacter {
    pub character_id: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveProfileUser {
    pub user_id: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveProfileSettingsSources {
    pub characters: Vec<EveProfileCharacter>,
    pub users: Vec<EveProfileUser>,
}
