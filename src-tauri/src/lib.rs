mod db;
mod dwm;
mod error;
mod eve_profile_tools;
mod hotkeys;
mod models;
mod thumbnail_service;
mod windows;

use std::sync::Arc;
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

use crate::db::DbService;
use crate::dwm::DwmService;
use crate::eve_profile_tools::EveProfileToolsService;
use crate::hotkeys::HotkeyService;
use crate::models::{
    ClientGroup, HealthSnapshot, MumbleLink, MumbleServerGroup, Profile, ThumbnailConfig,
    ThumbnailSetting,
};
use crate::thumbnail_service::ThumbnailService;
use crate::windows::WindowService;

pub struct AppState {
    db: Arc<DbService>,
    thumbnail_service: Arc<ThumbnailService>,
    hotkeys: Arc<HotkeyService>,
    window_service: Arc<WindowService>,
    dwm: Arc<DwmService>,
    eve_tools: Arc<EveProfileToolsService>,
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
        })
    }
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
    state.thumbnail_service.start(
        app_handle,
        state.db.clone(),
        state.window_service.clone(),
        state.dwm.clone(),
    );
    Ok(())
}

#[tauri::command]
fn update_profile_hotkey(
    state: State<'_, AppState>,
    profile_id: i64,
    hotkey: String,
) -> Result<(), String> {
    let normalized = state.hotkeys.validate_hotkey(&hotkey)?;
    state.db.update_profile_hotkey(profile_id, normalized)
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
) -> Result<(), String> {
    state.db.save_thumbnail_setting(profile_id, window_title, config)?;
    state.dwm.sync_thumbnail_graph();
    Ok(())
}

#[tauri::command]
fn get_client_groups(state: State<'_, AppState>, profile_id: i64) -> Result<Vec<ClientGroup>, String> {
    state.db.get_client_groups(profile_id)
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
        .update_client_group_hotkeys(group_id, forward, backward)
}

#[tauri::command]
fn cycle_client_group(
    state: State<'_, AppState>,
    group_id: i64,
    direction: String,
) -> Result<(), String> {
    let members = state.db.get_client_group_member_titles(group_id)?;
    if members.is_empty() {
        return Ok(());
    }

    let windows = state.window_service.enumerate_windows();
    let foreground_pid = state.window_service.foreground_window_pid();
    let current_title = windows
        .iter()
        .find(|w| Some(w.pid) == foreground_pid)
        .map(|w| w.title.clone());

    let step = if direction.eq_ignore_ascii_case("backward") {
        -1isize
    } else {
        1isize
    };

    let current_index = current_title
        .as_ref()
        .and_then(|title| members.iter().position(|m| m == title))
        .unwrap_or(0) as isize;
    let next_index = (current_index + step).rem_euclid(members.len() as isize) as usize;
    let next_title = &members[next_index];

    state.window_service.activate_window_by_title(next_title)?;
    Ok(())
}

#[tauri::command]
fn get_mumble_links(state: State<'_, AppState>) -> Result<Vec<MumbleLink>, String> {
    state.db.get_mumble_links()
}

#[tauri::command]
fn get_mumble_server_groups(state: State<'_, AppState>) -> Result<Vec<MumbleServerGroup>, String> {
    state.db.get_mumble_server_groups()
}

#[tauri::command]
fn get_app_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    state.db.get_app_setting(key)
}

#[tauri::command]
fn set_app_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    state.db.set_app_setting(key, value)
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
fn grid_apply_layout(state: State<'_, AppState>) -> Result<(), String> {
    state.window_service.apply_grid_layout();
    Ok(())
}

#[tauri::command]
fn eve_profiles_list(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.eve_tools.list_profiles())
}

#[tauri::command]
fn activate_window_by_pid(state: State<'_, AppState>, pid: u32) -> Result<(), String> {
    state.window_service.activate_window_by_pid(pid)
}

pub fn run() {
    let state = AppState::new().expect("failed to initialize app state");
    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let state = app.state::<AppState>();
            state.thumbnail_service.start(
                app.handle().clone(),
                state.db.clone(),
                state.window_service.clone(),
                state.dwm.clone(),
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
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
            get_client_groups,
            update_client_group_hotkeys,
            cycle_client_group,
            get_mumble_links,
            get_mumble_server_groups,
            get_app_setting,
            set_app_setting,
            hotkeys_capture_start,
            hotkeys_capture_stop,
            grid_apply_layout,
            eve_profiles_list,
            activate_window_by_pid
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
