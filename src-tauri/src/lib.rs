mod db;
mod diag;
mod dwm;
mod error;
mod eve_chat_log_service;
mod eve_profile_tools;
#[cfg(target_os = "windows")]
mod global_hotkeys;
mod hotkeys;
mod instance_guard;
mod models;
mod monitors;
mod thumbnail_service;
mod thumbnail_webview_overlay;
mod settings_backup;
mod widget_overlay;
mod widget_service;
mod windows;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WindowEvent};
#[cfg(target_os = "windows")]
use ::windows::core::HSTRING;
#[cfg(target_os = "windows")]
use ::windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

use crate::db::DbService;
use crate::dwm::DwmService;
use crate::eve_profile_tools::EveProfileToolsService;
use crate::hotkeys::HotkeyService;
use crate::models::{
    BrowserQuickLink, ClientGroup, ClientGroupDetail, DrawerSettings, EveChatChannel,
    EveDetectedProfile, EveLogSettings, EveProfileSettingsSources, GridLayoutFormPrefs,
    GridLayoutPayload, GridLayoutPreviewItem, HealthSnapshot, MonitorInfoDto, MumbleLink,
    MumbleLinksOverlaySettings,
    MumbleServerGroup, MumbleTreeSnapshot, Profile, ThumbnailConfig, ThumbnailSetting,
    WidgetOverlayLayout, WidgetOverlaySettings,
};
use crate::thumbnail_service::{RuntimeThumbnailStateSnapshot, ThumbnailService};
use crate::thumbnail_webview_overlay::ThumbnailOverlayStatePayload;
use crate::widget_overlay::WidgetOverlayHitRect;
use crate::widget_service::{WidgetService, WidgetSnapshot};
use crate::windows::WindowService;

const GENERIC_OPERATION_ERROR: &str = "Operation failed. Check diagnostics for details.";

pub struct AppState {
    db: Arc<DbService>,
    thumbnail_service: Arc<ThumbnailService>,
    hotkeys: Arc<HotkeyService>,
    window_service: Arc<WindowService>,
    dwm: Arc<DwmService>,
    eve_tools: Arc<EveProfileToolsService>,
    eve_chat_logs: Arc<eve_chat_log_service::EveChatLogService>,
    widget_service: Arc<WidgetService>,
}

impl AppState {
    fn new() -> Result<Self, String> {
        Ok(Self {
            db: Arc::new(DbService::new()?),
            thumbnail_service: Arc::new(ThumbnailService::default()),
            hotkeys: Arc::new(HotkeyService),
            window_service: Arc::new(WindowService),
            dwm: Arc::new(DwmService::default()),
            eve_tools: Arc::new(EveProfileToolsService),
            eve_chat_logs: Arc::new(eve_chat_log_service::EveChatLogService::default()),
            widget_service: Arc::new(WidgetService::default()),
        })
    }
}

fn sanitize_error(context: &str, err: String) -> String {
    eprintln!("{context}: {err}");
    GENERIC_OPERATION_ERROR.to_string()
}

fn is_boolean_setting_value(value: &str) -> bool {
    value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false")
}

fn validate_app_setting_key(key: &str) -> Result<(), String> {
    const ALLOWED_KEYS: &[&str] = &[
        "EnableThumbnailDragging",
        "StartHidden",
        "Theme",
        "DrawerScreenIndex",
        "DrawerHardwareId",
        "DrawerSide",
        "DrawerWidth",
        "DrawerHeight",
        "DrawerIsVisible",
        "DrawerIsEnabled",
        "DrawerSelectedMumbleServerGroupId",
    ];
    if ALLOWED_KEYS.contains(&key) {
        Ok(())
    } else {
        Err("Unknown setting key".to_string())
    }
}

fn validate_app_setting_value(key: &str, value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    match key {
        "EnableThumbnailDragging" | "StartHidden" | "DrawerIsVisible" | "DrawerIsEnabled" => {
            if !is_boolean_setting_value(trimmed) {
                return Err("Setting value must be true or false".to_string());
            }
        }
        "Theme" => {
            if !trimmed.eq_ignore_ascii_case("dark") && !trimmed.eq_ignore_ascii_case("light") {
                return Err("Theme must be Dark or Light".to_string());
            }
        }
        "DrawerScreenIndex" => {
            let parsed = trimmed
                .parse::<i64>()
                .map_err(|_| "Drawer screen index must be a number".to_string())?;
            if !(0..=32).contains(&parsed) {
                return Err("Drawer screen index is out of range".to_string());
            }
        }
        "DrawerWidth" => {
            let parsed = trimmed
                .parse::<i64>()
                .map_err(|_| "Drawer width must be a number".to_string())?;
            if !(200..=3000).contains(&parsed) {
                return Err("Drawer width is out of range".to_string());
            }
        }
        "DrawerHeight" => {
            let parsed = trimmed
                .parse::<i64>()
                .map_err(|_| "Drawer height must be a number".to_string())?;
            if !(150..=3000).contains(&parsed) {
                return Err("Drawer height is out of range".to_string());
            }
        }
        "DrawerSide" => {
            if !trimmed.eq_ignore_ascii_case("left") && !trimmed.eq_ignore_ascii_case("right") {
                return Err("Drawer side must be Left or Right".to_string());
            }
        }
        "DrawerSelectedMumbleServerGroupId" => {
            if !trimmed.is_empty() {
                let parsed = trimmed
                    .parse::<i64>()
                    .map_err(|_| "Drawer selected group id must be a number".to_string())?;
                if parsed <= 0 {
                    return Err("Drawer selected group id must be positive".to_string());
                }
            }
        }
        "DrawerHardwareId" => {
            if trimmed.len() > 256 {
                return Err("Drawer hardware id is too long".to_string());
            }
        }
        _ => {}
    }
    Ok(())
}

#[tauri::command]
fn list_monitors_cmd() -> Result<Vec<MonitorInfoDto>, String> {
    Ok(monitors::list_monitors())
}

#[tauri::command]
fn health(state: State<'_, AppState>) -> Result<HealthSnapshot, String> {
    Ok(HealthSnapshot {
        app: "yaep-rust",
        backend_ready: true,
        active_profile_id: state.db.active_profile_id(),
    })
}

#[tauri::command]
fn get_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    state.db.get_profiles()
}

#[tauri::command]
fn create_profile(state: State<'_, AppState>, name: String) -> Result<Profile, String> {
    state.db.create_profile(name)
}

#[tauri::command]
fn set_current_profile(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    profile_id: i64,
) -> Result<(), String> {
    state.db.set_active_profile(profile_id)?;
    state.thumbnail_service.stop();
    state.eve_chat_logs.stop();
    let app_for_thumbnails = app_handle.clone();
    state.thumbnail_service.start(
        app_for_thumbnails,
        state.db.clone(),
        state.window_service.clone(),
        state.dwm.clone(),
    );
    state
        .eve_chat_logs
        .start(app_handle, state.db.clone(), state.widget_service.clone());
    refresh_global_hotkeys();
    Ok(())
}

#[tauri::command]
fn update_profile_hotkey(
    state: State<'_, AppState>,
    profile_id: i64,
    hotkey: String,
) -> Result<(), String> {
    let normalized = state.hotkeys.validate_hotkey(&hotkey)?;
    state.db.update_profile_hotkey(profile_id, normalized)?;
    refresh_global_hotkeys();
    Ok(())
}

#[tauri::command]
fn delete_profile(state: State<'_, AppState>, profile_id: i64) -> Result<(), String> {
    state.db.delete_profile(profile_id)
}

#[tauri::command]
fn get_processes_to_preview(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<Vec<String>, String> {
    state.db.get_processes_to_preview(profile_id)
}

#[tauri::command]
fn add_process_to_preview(
    state: State<'_, AppState>,
    profile_id: i64,
    process_name: String,
) -> Result<(), String> {
    state.db.add_process_to_preview(profile_id, process_name)
}

#[tauri::command]
fn remove_process_to_preview(
    state: State<'_, AppState>,
    profile_id: i64,
    process_name: String,
) -> Result<(), String> {
    state.db.remove_process_to_preview(profile_id, process_name)
}

#[tauri::command]
fn get_thumbnail_default_config(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<ThumbnailConfig, String> {
    state.db.get_thumbnail_default_config(profile_id)
}

#[tauri::command]
fn set_thumbnail_default_config(
    state: State<'_, AppState>,
    profile_id: i64,
    config: ThumbnailConfig,
) -> Result<(), String> {
    state.db.set_thumbnail_default_config(profile_id, config)?;
    state.dwm.sync_thumbnail_graph();
    Ok(())
}

#[tauri::command]
fn get_thumbnail_settings(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<Vec<ThumbnailSetting>, String> {
    state.db.get_thumbnail_settings(profile_id)
}

#[tauri::command]
fn save_thumbnail_setting(
    state: State<'_, AppState>,
    profile_id: i64,
    window_title: String,
    config: ThumbnailConfig,
    character_id: Option<i64>,
) -> Result<(), String> {
    state
        .db
        .save_thumbnail_setting(profile_id, window_title, config, character_id)?;
    state.dwm.sync_thumbnail_graph();
    Ok(())
}

#[tauri::command]
fn eve_get_log_settings(state: State<'_, AppState>, profile_id: i64) -> Result<EveLogSettings, String> {
    state.db.get_eve_log_settings(profile_id)
}

#[tauri::command]
fn eve_save_log_settings(
    state: State<'_, AppState>,
    profile_id: i64,
    settings: EveLogSettings,
) -> Result<(), String> {
    state.db.save_eve_log_settings(profile_id, settings)
}

#[tauri::command]
fn eve_list_chat_channels(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<Vec<EveChatChannel>, String> {
    state.db.list_eve_chat_channels(profile_id)
}

#[tauri::command]
fn eve_add_chat_channel(
    state: State<'_, AppState>,
    profile_id: i64,
    channel_type: String,
    channel_name: String,
    background_color: Option<String>,
) -> Result<EveChatChannel, String> {
    state
        .db
        .add_eve_chat_channel(profile_id, channel_type, channel_name, background_color)
}

#[tauri::command]
fn eve_remove_chat_channel(
    state: State<'_, AppState>,
    profile_id: i64,
    channel_id: i64,
) -> Result<(), String> {
    state.db.remove_eve_chat_channel(profile_id, channel_id)
}

#[tauri::command]
fn eve_update_chat_channel_color(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    profile_id: i64,
    channel_id: i64,
    background_color: String,
) -> Result<(), String> {
    state
        .db
        .update_eve_chat_channel_color(profile_id, channel_id, background_color)?;
    let channels = state.db.list_eve_chat_channels(profile_id)?;
    let color_by_channel: std::collections::HashMap<String, String> = channels
        .into_iter()
        .map(|channel| (channel.channel_name, channel.background_color))
        .collect();
    state
        .widget_service
        .refresh_intel_channel_colors(&app_handle, &color_by_channel);
    Ok(())
}

#[tauri::command]
fn get_client_groups(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<Vec<ClientGroup>, String> {
    state.db.get_client_groups(profile_id)
}

#[tauri::command]
fn get_client_groups_detailed(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<Vec<ClientGroupDetail>, String> {
    state.db.get_client_groups_detailed(profile_id)
}

#[tauri::command]
fn create_client_group(
    state: State<'_, AppState>,
    profile_id: i64,
    name: String,
) -> Result<ClientGroupDetail, String> {
    state.db.create_client_group(profile_id, name)
}

#[tauri::command]
fn delete_client_group(
    state: State<'_, AppState>,
    profile_id: i64,
    group_id: i64,
) -> Result<(), String> {
    state.db.delete_client_group(profile_id, group_id)
}

#[tauri::command]
fn add_client_group_member(
    state: State<'_, AppState>,
    profile_id: i64,
    group_id: i64,
    window_title: String,
) -> Result<(), String> {
    state
        .db
        .add_client_group_member(profile_id, group_id, window_title)
}

#[tauri::command]
fn remove_client_group_member(
    state: State<'_, AppState>,
    profile_id: i64,
    group_id: i64,
    window_title: String,
) -> Result<(), String> {
    state
        .db
        .remove_client_group_member(profile_id, group_id, window_title)
}

#[tauri::command]
fn reorder_client_group_members(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    profile_id: i64,
    group_id: i64,
    window_titles_in_order: Vec<String>,
) -> Result<(), String> {
    let old_titles: Vec<String> = state
        .db
        .get_client_group_members(group_id)?
        .into_iter()
        .map(|m| m.window_title)
        .collect();
    let settings = state.db.get_thumbnail_settings(profile_id)?;
    let slot_geometry: Vec<Option<(i64, i64, i64, i64)>> = old_titles
        .iter()
        .map(|t| {
            settings
                .iter()
                .find(|s| s.window_title == *t)
                .map(|s| (s.config.x, s.config.y, s.config.width, s.config.height))
        })
        .collect();

    state.thumbnail_service.stop();
    state
        .db
        .reorder_client_group_members(profile_id, group_id, window_titles_in_order.clone())?;

    let ordered: Vec<String> = window_titles_in_order
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    for (j, new_title) in ordered.iter().enumerate() {
        let Some((x, y, w, h)) = slot_geometry.get(j).copied().flatten() else {
            continue;
        };
        let Some(current) = settings.iter().find(|s| s.window_title == *new_title) else {
            continue;
        };
        let mut next = current.config.clone();
        next.x = clamp_position(x);
        next.y = clamp_position(y);
        next.width = w;
        next.height = h;
        state
            .db
            .save_thumbnail_setting(profile_id, new_title.clone(), next, current.character_id)?;
    }

    state.thumbnail_service.start(
        app_handle,
        state.db.clone(),
        state.window_service.clone(),
        state.dwm.clone(),
    );
    state.window_service.apply_grid_layout(&state.dwm);
    Ok(())
}

#[tauri::command]
fn update_client_group_hotkeys(
    state: State<'_, AppState>,
    group_id: i64,
    cycle_forward_hotkey: String,
    cycle_backward_hotkey: String,
) -> Result<(), String> {
    let forward = state.hotkeys.validate_hotkey(&cycle_forward_hotkey)?;
    let backward = state.hotkeys.validate_hotkey(&cycle_backward_hotkey)?;
    state
        .db
        .update_client_group_hotkeys(group_id, forward, backward)?;
    refresh_global_hotkeys();
    Ok(())
}

pub(crate) fn cycle_client_group_internal(
    state: &AppState,
    group_id: i64,
    direction: &str,
    only_when_foreground_is_monitored: bool,
) -> Result<(), String> {
    if only_when_foreground_is_monitored
        && !state
            .thumbnail_service
            .is_foreground_a_runtime_thumbnail(&state.window_service)
    {
        if diag::enabled() {
            diag::trace(
                "hotkeys",
                "cycle_client_group: skipped (foreground not a PID in thumbnail runtime map)",
            );
        }
        return Ok(());
    }

    let mut members = state.db.get_client_group_member_titles(group_id)?;
    if members.is_empty() {
        return Ok(());
    }

    if only_when_foreground_is_monitored {
        members = state
            .thumbnail_service
            .filter_group_members_to_active_runtime(&members);
        if members.is_empty() {
            if diag::enabled() {
                diag::trace(
                    "hotkeys",
                    "cycle_client_group: skipped (no group members with active thumbnails)",
                );
            }
            return Ok(());
        }
    }

    let current_title = state
        .window_service
        .foreground_window_snapshot()
        .map(|w| w.title);

    let step = if direction.eq_ignore_ascii_case("backward") {
        -1isize
    } else {
        1isize
    };

    let current_index = current_title
        .as_ref()
        .and_then(|title| members.iter().position(|m| m.trim() == title.trim()))
        .unwrap_or(0) as isize;
    let next_index = (current_index + step).rem_euclid(members.len() as isize) as usize;
    let next_title = members[next_index].trim();

    if state
        .thumbnail_service
        .focus_thumbnail_client_by_title(next_title, &state.window_service)
        .is_ok()
    {
        return Ok(());
    }

    if only_when_foreground_is_monitored {
        if diag::enabled() {
            diag::trace(
                "hotkeys",
                "cycle_client_group: focus failed for active member (stale snapshot), skipping",
            );
        }
        return Ok(());
    }

    state.window_service.activate_window_by_title(next_title)?;
    Ok(())
}

#[tauri::command]
fn cycle_client_group(
    state: State<'_, AppState>,
    group_id: i64,
    direction: String,
) -> Result<(), String> {
    cycle_client_group_internal(&state, group_id, &direction, false)
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.len() > 2048 {
        return Err("Invalid URL".to_string());
    }
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return Err("Only http/https URLs are allowed".to_string());
    }
    opener::open(trimmed).map_err(|e| sanitize_error("open_external_url", e.to_string()))?;
    Ok(())
}

pub(crate) fn open_mumble_link_internal(state: &AppState, link_id: i64) -> Result<(), String> {
    let links = state.db.get_mumble_links()?;
    let link = links
        .into_iter()
        .find(|l| l.id == link_id)
        .ok_or_else(|| "Mumble link not found".to_string())?;
    opener::open(&link.url).map_err(|e| sanitize_error("open_mumble_link", e.to_string()))?;
    Ok(())
}

fn emit_mumble_tree_changed(app: &AppHandle) {
    let _ = app.emit("mumble-tree-changed", ());
}

#[tauri::command]
fn open_mumble_link(state: State<'_, AppState>, link_id: i64) -> Result<(), String> {
    open_mumble_link_internal(&state, link_id)
}

#[cfg(target_os = "windows")]
pub(crate) fn refresh_global_hotkeys() {
    global_hotkeys::request_refresh();
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn refresh_global_hotkeys() {}

#[tauri::command]
fn get_mumble_links(state: State<'_, AppState>) -> Result<Vec<MumbleLink>, String> {
    state.db.get_mumble_links()
}

#[tauri::command]
fn get_mumble_tree(state: State<'_, AppState>) -> Result<MumbleTreeSnapshot, String> {
    state.db.get_mumble_tree()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateMumbleFolderPayload {
    server_group_id: i64,
    parent_folder_id: Option<i64>,
    name: String,
    display_order: i64,
    #[serde(default)]
    icon_key: Option<String>,
}

#[tauri::command]
fn create_mumble_folder(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    payload: CreateMumbleFolderPayload,
) -> Result<i64, String> {
    let id = state.db.create_mumble_folder(
        payload.server_group_id,
        payload.parent_folder_id,
        payload.name,
        payload.display_order,
        payload.icon_key,
    )?;
    emit_mumble_tree_changed(&app_handle);
    Ok(id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMumbleFolderPayload {
    folder_id: i64,
    name: String,
    display_order: i64,
    #[serde(default)]
    icon_key: Option<String>,
}

#[tauri::command]
fn update_mumble_folder(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    payload: UpdateMumbleFolderPayload,
) -> Result<(), String> {
    state
        .db
        .update_mumble_folder(
            payload.folder_id,
            payload.name,
            payload.display_order,
            payload.icon_key,
        )?;
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[tauri::command]
fn delete_mumble_folder(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    folder_id: i64,
) -> Result<(), String> {
    state.db.delete_mumble_folder(folder_id)?;
    refresh_global_hotkeys();
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateMumbleLinkPayload {
    name: String,
    url: String,
    display_order: i64,
    hotkey: String,
    server_group_id: i64,
    folder_id: Option<i64>,
}

#[tauri::command]
fn create_mumble_link(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    payload: CreateMumbleLinkPayload,
) -> Result<(), String> {
    let normalized_hotkey = state.hotkeys.validate_hotkey(&payload.hotkey)?;
    state.db.create_mumble_link(
        payload.name,
        payload.url,
        payload.display_order,
        normalized_hotkey,
        payload.server_group_id,
        payload.folder_id,
    )?;
    refresh_global_hotkeys();
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMumbleLinkPayload {
    link_id: i64,
    name: String,
    url: String,
    display_order: i64,
    hotkey: String,
    server_group_id: i64,
    folder_id: Option<i64>,
}

#[tauri::command]
fn update_mumble_link(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    payload: UpdateMumbleLinkPayload,
) -> Result<(), String> {
    let normalized_hotkey = state.hotkeys.validate_hotkey(&payload.hotkey)?;
    state.db.update_mumble_link(
        payload.link_id,
        payload.name,
        payload.url,
        payload.display_order,
        normalized_hotkey,
        payload.server_group_id,
        payload.folder_id,
    )?;
    refresh_global_hotkeys();
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[tauri::command]
fn set_mumble_link_selected(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    link_id: i64,
    is_selected: bool,
) -> Result<(), String> {
    state.db.set_mumble_link_selected(link_id, is_selected)?;
    refresh_global_hotkeys();
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[tauri::command]
fn delete_mumble_link(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    link_id: i64,
) -> Result<(), String> {
    state.db.delete_mumble_link(link_id)?;
    refresh_global_hotkeys();
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[tauri::command]
fn get_mumble_server_groups(state: State<'_, AppState>) -> Result<Vec<MumbleServerGroup>, String> {
    state.db.get_mumble_server_groups()
}

#[tauri::command]
fn create_mumble_server_group(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    name: String,
    display_order: i64,
) -> Result<(), String> {
    state.db.create_mumble_server_group(name, display_order)?;
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[tauri::command]
fn update_mumble_server_group(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    group_id: i64,
    name: String,
    display_order: i64,
) -> Result<(), String> {
    state
        .db
        .update_mumble_server_group(group_id, name, display_order)?;
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[tauri::command]
fn delete_mumble_server_group(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    group_id: i64,
) -> Result<(), String> {
    state.db.delete_mumble_server_group(group_id)?;
    refresh_global_hotkeys();
    emit_mumble_tree_changed(&app_handle);
    Ok(())
}

#[tauri::command]
fn get_mumble_links_overlay_settings(
    state: State<'_, AppState>,
) -> Result<MumbleLinksOverlaySettings, String> {
    state.db.get_mumble_links_overlay_settings()
}

#[tauri::command]
fn save_mumble_links_overlay_settings(
    state: State<'_, AppState>,
    settings: MumbleLinksOverlaySettings,
) -> Result<(), String> {
    state.db.save_mumble_links_overlay_settings(settings)?;
    Ok(())
}

#[tauri::command]
fn get_drawer_settings(state: State<'_, AppState>) -> Result<DrawerSettings, String> {
    state.db.get_drawer_settings()
}

#[tauri::command]
fn save_drawer_settings(
    state: State<'_, AppState>,
    settings: DrawerSettings,
) -> Result<(), String> {
    state.db.save_drawer_settings(settings)?;
    Ok(())
}

#[tauri::command]
fn get_app_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    validate_app_setting_key(&key)?;
    state
        .db
        .get_app_setting(key)
        .map_err(|e| sanitize_error("get_app_setting", e))
}

#[tauri::command]
fn set_app_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    validate_app_setting_key(&key)?;
    validate_app_setting_value(&key, &value)?;
    state
        .db
        .set_app_setting(key, value)
        .map_err(|e| sanitize_error("set_app_setting", e))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyCapturePayload {
    capture_type: String,
    target_id: Option<i64>,
}

#[tauri::command]
fn hotkeys_capture_start(
    state: State<'_, AppState>,
    payload: HotkeyCapturePayload,
) -> Result<(), String> {
    state
        .hotkeys
        .capture_start(payload.capture_type, payload.target_id);
    Ok(())
}

#[tauri::command]
fn hotkeys_capture_stop(state: State<'_, AppState>) -> Result<(), String> {
    state.hotkeys.capture_stop();
    Ok(())
}

#[tauri::command]
fn grid_preview_layout(
    state: State<'_, AppState>,
    payload: GridLayoutPayload,
) -> Result<Vec<GridLayoutPreviewItem>, String> {
    build_grid_layout_preview(&state, payload)
}

#[tauri::command]
fn grid_apply_layout(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    payload: GridLayoutPayload,
) -> Result<(), String> {
    let plan = build_grid_layout_preview(&state, payload.clone())?;
    let existing = state.db.get_thumbnail_settings(payload.profile_id)?;

    state.thumbnail_service.stop();
    for item in plan {
        let Some(current) = existing
            .iter()
            .find(|setting| setting.window_title == item.window_title)
        else {
            continue;
        };
        let mut next = current.config.clone();
        next.width = item.width;
        next.height = item.height;
        let (fx, fy) = if let Some(mi) = payload.selected_monitor_index {
            monitors::clamp_rect_to_monitor_work_area(
                mi,
                item.x as i32,
                item.y as i32,
                item.width as i32,
                item.height as i32,
            )
            .unwrap_or((item.x as i32, item.y as i32))
        } else {
            (item.x as i32, item.y as i32)
        };
        next.x = clamp_position(fx as i64);
        next.y = clamp_position(fy as i64);
        state
            .db
            .save_thumbnail_setting(payload.profile_id, item.window_title.clone(), next, None)?;
    }
    state.thumbnail_service.start(
        app_handle,
        state.db.clone(),
        state.window_service.clone(),
        state.dwm.clone(),
    );

    state.window_service.apply_grid_layout(&state.dwm);
    Ok(())
}

#[tauri::command]
fn grid_layout_get_prefs(
    state: State<'_, AppState>,
    profile_id: i64,
) -> Result<Option<GridLayoutFormPrefs>, String> {
    state
        .db
        .get_grid_layout_prefs(profile_id)
        .map_err(|e| sanitize_error("grid_layout_get_prefs", e))
}

#[tauri::command]
fn grid_layout_save_prefs(
    state: State<'_, AppState>,
    profile_id: i64,
    prefs: GridLayoutFormPrefs,
) -> Result<(), String> {
    state
        .db
        .set_grid_layout_prefs(profile_id, prefs)
        .map_err(|e| sanitize_error("grid_layout_save_prefs", e))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GridLayoutPrefsExportBody {
    export_kind: &'static str,
    version: u32,
    profile_id: i64,
    prefs: GridLayoutFormPrefs,
}

#[tauri::command]
fn grid_layout_export_prefs_to_path(
    path: String,
    profile_id: i64,
    mut prefs: GridLayoutFormPrefs,
) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("No file path provided.".to_string());
    }
    prefs.normalize();
    let body = GridLayoutPrefsExportBody {
        export_kind: "yaep-grid-layout-prefs",
        version: 1,
        profile_id,
        prefs,
    };
    let json = serde_json::to_string_pretty(&body).map_err(|e| e.to_string())?;
    std::fs::write(trimmed, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn eve_profiles_list(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.eve_tools.list_profiles()
}

#[tauri::command]
fn eve_profiles_detected(state: State<'_, AppState>) -> Result<Vec<EveDetectedProfile>, String> {
    state.eve_tools.list_detected_profiles()
}

#[tauri::command]
fn eve_copy_profile_on_server(
    state: State<'_, AppState>,
    server_name: String,
    source_profile_name: String,
    new_profile_name: String,
) -> Result<(), String> {
    state
        .eve_tools
        .copy_profile_on_server(server_name, source_profile_name, new_profile_name)
}

#[tauri::command]
fn eve_delete_profile_on_server(
    state: State<'_, AppState>,
    server_name: String,
    profile_name: String,
) -> Result<(), String> {
    state.eve_tools.delete_profile_on_server(server_name, profile_name)
}

#[tauri::command]
fn eve_get_profile_settings_sources(
    state: State<'_, AppState>,
    server_name: String,
    profile_name: String,
) -> Result<EveProfileSettingsSources, String> {
    state
        .eve_tools
        .get_profile_settings_sources(server_name, profile_name)
}

#[tauri::command]
fn eve_copy_profile_settings_from_sources(
    state: State<'_, AppState>,
    server_name: String,
    profile_name: String,
    source_character_id: String,
    source_user_id: String,
) -> Result<(), String> {
    state.eve_tools.copy_profile_settings_from_sources(
        server_name,
        profile_name,
        source_character_id,
        source_user_id,
    )
}

#[tauri::command]
fn eve_copy_profile(
    state: State<'_, AppState>,
    source_profile: String,
    new_profile: String,
) -> Result<(), String> {
    state.eve_tools.copy_profile(source_profile, new_profile)
}

#[tauri::command]
fn eve_copy_character_files(
    state: State<'_, AppState>,
    source_profile: String,
    target_profile: String,
) -> Result<(), String> {
    state
        .eve_tools
        .copy_character_files(source_profile, target_profile)
}

#[tauri::command]
fn eve_copy_character_files_on_server(
    state: State<'_, AppState>,
    server_name: String,
    source_profile_name: String,
    target_profile_name: String,
) -> Result<(), String> {
    state.eve_tools.copy_character_files_on_server(
        server_name,
        source_profile_name,
        target_profile_name,
    )
}

#[tauri::command]
fn eve_backup_all_profiles(
    state: State<'_, AppState>,
    server_name: String,
    output_path: String,
) -> Result<(), String> {
    state
        .eve_tools
        .backup_all_profiles(server_name, output_path)
        .map_err(|e| sanitize_error("eve_backup_all_profiles", e))
}

#[tauri::command]
fn eve_fetch_character_name(
    state: State<'_, AppState>,
    character_id: u64,
) -> Result<String, String> {
    state.eve_tools.fetch_character_name(character_id)
}

#[tauri::command]
fn activate_window_by_pid(state: State<'_, AppState>, pid: u32) -> Result<(), String> {
    let snapshot = state.thumbnail_service.snapshot_state();
    if !snapshot.thumbnails.iter().any(|thumb| thumb.pid == pid) {
        return Err("Only active runtime thumbnails can be activated".to_string());
    }
    state.window_service.activate_window_by_pid(pid)
}

#[tauri::command]
fn get_runtime_thumbnail_state(
    state: State<'_, AppState>,
) -> Result<RuntimeThumbnailStateSnapshot, String> {
    Ok(state.thumbnail_service.snapshot_state())
}

#[tauri::command]
fn widget_get_snapshot(state: State<'_, AppState>) -> Result<WidgetSnapshot, String> {
    Ok(state.widget_service.snapshot())
}

#[tauri::command]
fn app_ready(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    let thumbnail_service = state.thumbnail_service.clone();
    let db = state.db.clone();
    let db_for_logs = state.db.clone();
    let window_service = state.window_service.clone();
    let dwm = state.dwm.clone();
    let eve_chat_logs = state.eve_chat_logs.clone();
    let widget_service = state.widget_service.clone();
    tauri::async_runtime::spawn(async move {
        let app_for_thumbnails = app_handle.clone();
        thumbnail_service.stop();
        thumbnail_service.start(app_for_thumbnails, db, window_service, dwm);
        eve_chat_logs.stop();
        eve_chat_logs.start(app_handle, db_for_logs, widget_service);
    });
    Ok(())
}

#[tauri::command]
fn get_thumbnail_overlay_state(
    state: State<'_, AppState>,
    overlay_id: String,
) -> Option<ThumbnailOverlayStatePayload> {
    state.dwm.snapshot_thumbnail_overlay_state(&overlay_id)
}

#[tauri::command]
fn widget_overlay_get_settings(state: State<'_, AppState>) -> Result<WidgetOverlaySettings, String> {
    widget_overlay::load_settings(&state.db)
}

#[tauri::command]
fn widget_overlay_save_settings(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    settings: WidgetOverlaySettings,
) -> Result<(), String> {
    widget_overlay::save_settings(&state.db, &settings)?;
    widget_overlay::sync_widget_overlay_from_db(&app_handle, &state.db);
    refresh_global_hotkeys();
    let _ = app_handle.emit("widget-overlay-settings-changed", ());
    Ok(())
}

#[tauri::command]
fn widget_overlay_save_layout(state: State<'_, AppState>, layout: WidgetOverlayLayout) -> Result<(), String> {
    widget_overlay::save_layout_only(&state.db, &layout)
}

#[tauri::command]
fn widget_overlay_save_browser_quick_links(
    state: State<'_, AppState>,
    links: Vec<BrowserQuickLink>,
    default_url: Option<String>,
) -> Result<(), String> {
    widget_overlay::save_browser_quick_links_only(&state.db, &links, default_url.as_deref())
}

#[tauri::command]
fn widget_overlay_refresh(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    widget_overlay::sync_widget_overlay_from_db(&app_handle, &state.db);
    Ok(())
}

#[tauri::command]
fn widget_overlay_update_hit_regions(rects: Vec<WidgetOverlayHitRect>) {
    widget_overlay::update_hit_regions(rects);
}

#[tauri::command]
fn widget_overlay_set_dragging(dragging: bool) {
    widget_overlay::set_dragging(dragging);
}

/// Toggles `widgets_suppressed` (hide non-pinned widgets). Overlay window stays open.
#[tauri::command]
fn widget_overlay_toggle(state: State<'_, AppState>, app_handle: AppHandle) -> Result<bool, String> {
    widget_overlay::toggle_widgets_suppressed(&app_handle, state.db.as_ref())
}

fn refresh_runtime_after_settings_import(state: &AppState, app_handle: &AppHandle) {
    state.thumbnail_service.stop();
    state.eve_chat_logs.stop();
    let app_for_thumbnails = app_handle.clone();
    state.thumbnail_service.start(
        app_for_thumbnails,
        state.db.clone(),
        state.window_service.clone(),
        state.dwm.clone(),
    );
    state.eve_chat_logs.start(
        app_handle.clone(),
        state.db.clone(),
        state.widget_service.clone(),
    );
    state.dwm.sync_thumbnail_graph();
    refresh_global_hotkeys();
    widget_overlay::sync_widget_overlay_from_db(app_handle, state.db.as_ref());
    let _ = app_handle.emit("mumble-tree-changed", ());
    let _ = app_handle.emit("widget-overlay-settings-changed", ());
}

#[tauri::command]
fn yaep_export_settings(state: State<'_, AppState>) -> Result<String, String> {
    crate::settings_backup::export_bundle(state.db.as_ref())
}

#[tauri::command]
fn yaep_export_settings_to_path(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("No file path provided.".to_string());
    }
    let json = crate::settings_backup::export_bundle(state.db.as_ref())?;
    std::fs::write(trimmed, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn yaep_import_settings(state: State<'_, AppState>, app_handle: AppHandle, json: String) -> Result<(), String> {
    crate::settings_backup::import_bundle(state.db.as_ref(), &json)?;
    refresh_runtime_after_settings_import(&state, &app_handle);
    Ok(())
}

#[tauri::command]
fn yaep_import_settings_from_path(state: State<'_, AppState>, app_handle: AppHandle, path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("No file path provided.".to_string());
    }
    let json = std::fs::read_to_string(trimmed).map_err(|e| e.to_string())?;
    crate::settings_backup::import_bundle(state.db.as_ref(), &json)?;
    refresh_runtime_after_settings_import(&state, &app_handle);
    Ok(())
}

pub fn run() {
    diag::install_panic_hook();
    diag::trace("boot", "run() entered");
    #[cfg(target_os = "windows")]
    set_windows_app_user_model_id();
    if let Err(error) = instance_guard::ensure_single_instance() {
        eprintln!("single-instance check failed: {error}");
        return;
    }
    diag::trace("boot", "single-instance ok");

    let state = match AppState::new() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("failed to initialize app state: {error}");
            return;
        }
    };
    diag::trace("boot", "AppState initialized");

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(|app| {
            diag::trace("boot", "tauri setup callback start");
            if let Err(error) = setup_tray(app) {
                eprintln!("tray setup failed: {error}");
            }
            diag::trace("boot", "tray setup finished");
            #[cfg(target_os = "windows")]
            {
                diag::trace("boot", "global_hotkeys: set_app_handle + spawn_thread");
                let db_for_hotkeys = app.state::<AppState>().db.clone();
                global_hotkeys::set_app_handle(&app.handle());
                global_hotkeys::spawn_thread(db_for_hotkeys);
                global_hotkeys::request_refresh();
                diag::trace("boot", "global_hotkeys scheduled");
            }
            #[cfg(target_os = "windows")]
            {
                let db = app.state::<AppState>().db.clone();
                if let Ok(hidden) = db.get_app_setting("StartHidden".to_string()) {
                    if hidden.is_some_and(|v| v.eq_ignore_ascii_case("true")) {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                }
            }
            {
                let db = app.state::<AppState>().db.clone();
                widget_overlay::ensure_cursor_poll(app.handle());
                widget_overlay::sync_widget_overlay_from_db(app.handle(), &db);
            }
            diag::trace("boot", "tauri setup callback complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            list_monitors_cmd,
            get_profiles,
            create_profile,
            set_current_profile,
            update_profile_hotkey,
            delete_profile,
            get_processes_to_preview,
            add_process_to_preview,
            remove_process_to_preview,
            get_thumbnail_default_config,
            set_thumbnail_default_config,
            get_thumbnail_settings,
            save_thumbnail_setting,
            eve_get_log_settings,
            eve_save_log_settings,
            eve_list_chat_channels,
            eve_add_chat_channel,
            eve_remove_chat_channel,
            eve_update_chat_channel_color,
            get_client_groups,
            get_client_groups_detailed,
            create_client_group,
            delete_client_group,
            add_client_group_member,
            remove_client_group_member,
            reorder_client_group_members,
            update_client_group_hotkeys,
            cycle_client_group,
            get_mumble_links,
            get_mumble_tree,
            create_mumble_folder,
            update_mumble_folder,
            delete_mumble_folder,
            create_mumble_link,
            update_mumble_link,
            open_mumble_link,
            set_mumble_link_selected,
            delete_mumble_link,
            get_mumble_server_groups,
            create_mumble_server_group,
            update_mumble_server_group,
            delete_mumble_server_group,
            get_mumble_links_overlay_settings,
            save_mumble_links_overlay_settings,
            get_drawer_settings,
            save_drawer_settings,
            get_app_setting,
            set_app_setting,
            hotkeys_capture_start,
            hotkeys_capture_stop,
            grid_preview_layout,
            grid_apply_layout,
            grid_layout_get_prefs,
            grid_layout_save_prefs,
            grid_layout_export_prefs_to_path,
            eve_profiles_list,
            eve_profiles_detected,
            eve_copy_profile_on_server,
            eve_delete_profile_on_server,
            eve_get_profile_settings_sources,
            eve_copy_profile_settings_from_sources,
            eve_copy_profile,
            eve_copy_character_files,
            eve_copy_character_files_on_server,
            eve_backup_all_profiles,
            eve_fetch_character_name,
            activate_window_by_pid,
            open_external_url,
            app_ready,
            widget_get_snapshot,
            get_runtime_thumbnail_state,
            get_thumbnail_overlay_state,
            widget_overlay_get_settings,
            widget_overlay_save_settings,
            widget_overlay_save_layout,
            widget_overlay_save_browser_quick_links,
            widget_overlay_refresh,
            widget_overlay_update_hit_regions,
            widget_overlay_set_dragging,
            widget_overlay_toggle,
            yaep_export_settings,
            yaep_export_settings_to_path,
            yaep_import_settings,
            yaep_import_settings_from_path
        ])
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            let WindowEvent::CloseRequested { api, .. } = event else {
                return;
            };
            api.prevent_close();
            let _ = window.hide();
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("failed to run tauri application: {error}");
    }
}

#[cfg(target_os = "windows")]
fn set_windows_app_user_model_id() {
    // Ensure Windows notifications are attributed to YAEP instead of the shell host process.
    let app_id = HSTRING::from("com.yaep.rust");
    if let Err(error) = unsafe { SetCurrentProcessExplicitAppUserModelID(&app_id) } {
        eprintln!("failed to set AppUserModelID: {error}");
    }
}

/// Tray menu callbacks can run in a re-entrant event-loop context on Windows; showing the
/// webview window synchronously often fails. Queue on the main loop via a fresh thread so
/// `run_on_main_thread` uses the proxy path (next event-loop turn).
fn show_main_window_from_tray<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(target_os = "windows")]
    {
        let app_h = app.clone();
        std::thread::spawn(move || {
            let h = app_h.clone();
            if let Err(e) = app_h.run_on_main_thread(move || {
                if let Some(window) = h.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }) {
                eprintln!("tray Show: run_on_main_thread failed: {e}");
            }
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let profiles = state.db.get_profiles().unwrap_or_default();

    let mut menu_builder = MenuBuilder::new(app);
    let show_item = MenuItemBuilder::new("Show").id("tray.show").build(app)?;
    let widget_overlay_toggle_item = MenuItemBuilder::new("Toggle widget visibility")
        .id("tray.widget_overlay_toggle")
        .build(app)?;
    menu_builder = menu_builder.item(&show_item).item(&widget_overlay_toggle_item);

    for profile in profiles {
        let label = if profile.is_active {
            format!("* {}", profile.name)
        } else {
            profile.name
        };
        let item = MenuItemBuilder::new(label)
            .id(format!("tray.profile.{}", profile.id))
            .build(app)?;
        menu_builder = menu_builder.item(&item);
    }

    let exit_item = MenuItemBuilder::new("Exit").id("tray.exit").build(app)?;
    let menu = menu_builder.item(&exit_item).build()?;

    let app_handle = app.handle().clone();
    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip(app.package_info().name.clone());
    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    } else {
        eprintln!(
            "tray: no default_window_icon; add icons under src-tauri/icons (see tauri.conf.json bundle.icon)"
        );
    }
    let tray = tray_builder
        .on_menu_event(move |app, event| {
            let id = event.id.as_ref();
            if id == "tray.show" {
                show_main_window_from_tray(app);
                return;
            }
            if id == "tray.widget_overlay_toggle" {
                let state = app.state::<AppState>();
                if let Err(e) = crate::widget_overlay::toggle_widgets_suppressed(app, state.db.as_ref()) {
                    eprintln!("tray: widget overlay toggle: {e}");
                }
                return;
            }
            if id == "tray.exit" {
                app.exit(0);
                return;
            }
            if let Some(profile_suffix) = id.strip_prefix("tray.profile.") {
                if let Ok(profile_id) = profile_suffix.parse::<i64>() {
                    let state = app.state::<AppState>();
                    if state.db.set_active_profile(profile_id).is_ok() {
                        state.thumbnail_service.stop();
                        state.thumbnail_service.start(
                            app_handle.clone(),
                            state.db.clone(),
                            state.window_service.clone(),
                            state.dwm.clone(),
                        );
                        let _ = app.emit(
                            "profileChanged",
                            serde_json::json!({ "profileId": profile_id }),
                        );
                        refresh_global_hotkeys();
                    }
                }
            }
        })
        .build(app)?;

    // Keep tray icon alive for app lifetime.
    std::mem::forget(tray);
    Ok(())
}

fn build_grid_layout_preview(
    state: &State<'_, AppState>,
    payload: GridLayoutPayload,
) -> Result<Vec<GridLayoutPreviewItem>, String> {
    if payload.grid_columns <= 0 {
        return Err("Grid columns must be greater than zero".to_string());
    }
    if payload.grid_cell_width <= 0 {
        return Err("Grid cell width must be greater than zero".to_string());
    }

    let mut settings = state.db.get_thumbnail_settings(payload.profile_id)?;
    if let Some(group_id) = payload.selected_group_id {
        let titles = state.db.get_client_group_member_titles(group_id)?;
        settings.retain(|setting| {
            let st = setting.window_title.trim();
            titles.iter().any(|t| t.trim() == st)
        });
        settings.sort_by_key(|setting| {
            let st = setting.window_title.trim();
            titles
                .iter()
                .position(|title| title.trim() == st)
                .unwrap_or(usize::MAX)
        });
    } else {
        let combined = state
            .db
            .client_group_combined_member_order(payload.profile_id)?;
        settings.sort_by(|a, b| {
            let at = a.window_title.trim();
            let bt = b.window_title.trim();
            let pa = combined.iter().position(|t| t == at);
            let pb = combined.iter().position(|t| t == bt);
            match (pa, pb) {
                (Some(ia), Some(ib)) => ia.cmp(&ib),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a
                    .config
                    .x
                    .cmp(&b.config.x)
                    .then_with(|| a.window_title.cmp(&b.window_title)),
            }
        });
    }

    if payload.only_affect_active_thumbnails {
        let active_titles: Vec<String> = state
            .window_service
            .enumerate_windows()
            .into_iter()
            .map(|w| w.title)
            .collect();
        settings.retain(|setting| {
            let st = setting.window_title.trim();
            active_titles.iter().any(|t| t.trim() == st)
        });
    }

    let cell_height = resolve_grid_cell_height(
        payload.grid_cell_width,
        payload.grid_cell_height,
        payload.grid_cell_ratio,
    )?;
    let (ox, oy) = payload
        .selected_monitor_index
        .and_then(monitors::work_area_offset)
        .unwrap_or((0, 0));

    let (start_x, start_y) = if let Some(ref anchor_title) = payload.grid_anchor_window_title {
        let idx = settings
            .iter()
            .position(|s| s.window_title.trim() == anchor_title.trim())
            .ok_or_else(|| {
                format!(
                    "Initial thumbnail not in grid scope (title missing or filtered out): {}",
                    anchor_title
                )
            })?;
        let anchor = settings.remove(idx);
        let ax = anchor.config.x;
        let ay = anchor.config.y;
        settings.insert(0, anchor);
        (ax.saturating_sub(ox as i64), ay.saturating_sub(oy as i64))
    } else {
        (payload.grid_start_x, payload.grid_start_y)
    };

    let mut preview = Vec::new();
    for (index, setting) in settings.iter().enumerate() {
        let col = (index as i64) % payload.grid_columns;
        let row = (index as i64) / payload.grid_columns;
        preview.push(GridLayoutPreviewItem {
            window_title: setting.window_title.clone(),
            x: start_x + (col * payload.grid_cell_width) + ox as i64,
            y: start_y + (row * cell_height) + oy as i64,
            width: payload.grid_cell_width,
            height: cell_height,
        });
    }
    Ok(preview)
}

fn resolve_grid_cell_height(
    grid_cell_width: i64,
    grid_cell_height: Option<i64>,
    grid_cell_ratio: Option<String>,
) -> Result<i64, String> {
    if let Some(height) = grid_cell_height {
        if height > 0 {
            return Ok(height);
        }
        return Err("Grid cell height must be greater than zero".to_string());
    }

    if let Some(ratio) = grid_cell_ratio {
        let trimmed = ratio.trim();
        if trimmed.is_empty() {
            return Err("Grid ratio cannot be empty".to_string());
        }
        let (left, right) = trimmed
            .split_once(':')
            .ok_or_else(|| "Grid ratio must use w:h format".to_string())?;
        let w = left
            .trim()
            .parse::<f64>()
            .map_err(|_| "Invalid ratio width".to_string())?;
        let h = right
            .trim()
            .parse::<f64>()
            .map_err(|_| "Invalid ratio height".to_string())?;
        if w <= 0.0 || h <= 0.0 {
            return Err("Grid ratio values must be greater than zero".to_string());
        }
        return Ok(((grid_cell_width as f64) * (h / w)).round() as i64);
    }

    Ok(300)
}

fn clamp_position(value: i64) -> i64 {
    value.clamp(-10_000, 31_000)
}

#[cfg(test)]
mod tests {
    use super::{clamp_position, resolve_grid_cell_height};

    #[test]
    fn resolve_grid_cell_height_uses_explicit_height_when_valid() {
        let height =
            resolve_grid_cell_height(600, Some(240), None).expect("expected explicit height");
        assert_eq!(height, 240);
    }

    #[test]
    fn resolve_grid_cell_height_rejects_non_positive_explicit_height() {
        let error =
            resolve_grid_cell_height(600, Some(0), None).expect_err("expected validation error");
        assert_eq!(error, "Grid cell height must be greater than zero");
    }

    #[test]
    fn resolve_grid_cell_height_parses_ratio_and_rounds_result() {
        let height = resolve_grid_cell_height(500, None, Some("16:9".to_string()))
            .expect("expected ratio-derived height");
        assert_eq!(height, 281);
    }

    #[test]
    fn resolve_grid_cell_height_rejects_invalid_ratio_format() {
        let error = resolve_grid_cell_height(500, None, Some("16x9".to_string()))
            .expect_err("expected format error");
        assert_eq!(error, "Grid ratio must use w:h format");
    }

    #[test]
    fn clamp_position_limits_values_to_supported_bounds() {
        assert_eq!(clamp_position(-20_000), -10_000);
        assert_eq!(clamp_position(42), 42);
        assert_eq!(clamp_position(40_000), 31_000);
    }
}
