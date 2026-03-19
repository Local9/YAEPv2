mod db;
mod dwm;
mod error;
mod eve_profile_tools;
mod hotkeys;
mod instance_guard;
mod models;
mod thumbnail_service;
mod windows;

use std::sync::Arc;
use serde::Deserialize;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};

use crate::db::DbService;
use crate::dwm::DwmService;
use crate::eve_profile_tools::EveProfileToolsService;
use crate::hotkeys::HotkeyService;
use crate::models::{
    ClientGroup, DrawerSettings, GridLayoutPayload, GridLayoutPreviewItem, HealthSnapshot, MumbleLink,
    MumbleLinksOverlaySettings, MumbleServerGroup, Profile, ThumbnailConfig, ThumbnailSetting,
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
fn create_mumble_link(
    state: State<'_, AppState>,
    name: String,
    url: String,
    display_order: i64,
    hotkey: String,
) -> Result<(), String> {
    let normalized_hotkey = state.hotkeys.validate_hotkey(&hotkey)?;
    state
        .db
        .create_mumble_link(name, url, display_order, normalized_hotkey)
}

#[tauri::command]
fn update_mumble_link(
    state: State<'_, AppState>,
    link_id: i64,
    name: String,
    url: String,
    display_order: i64,
    hotkey: String,
) -> Result<(), String> {
    let normalized_hotkey = state.hotkeys.validate_hotkey(&hotkey)?;
    state
        .db
        .update_mumble_link(link_id, name, url, display_order, normalized_hotkey)
}

#[tauri::command]
fn set_mumble_link_selected(
    state: State<'_, AppState>,
    link_id: i64,
    is_selected: bool,
) -> Result<(), String> {
    state.db.set_mumble_link_selected(link_id, is_selected)
}

#[tauri::command]
fn delete_mumble_link(state: State<'_, AppState>, link_id: i64) -> Result<(), String> {
    state.db.delete_mumble_link(link_id)
}

#[tauri::command]
fn get_mumble_server_groups(state: State<'_, AppState>) -> Result<Vec<MumbleServerGroup>, String> {
    state.db.get_mumble_server_groups()
}

#[tauri::command]
fn create_mumble_server_group(
    state: State<'_, AppState>,
    name: String,
    display_order: i64,
) -> Result<(), String> {
    state.db.create_mumble_server_group(name, display_order)
}

#[tauri::command]
fn update_mumble_server_group(
    state: State<'_, AppState>,
    group_id: i64,
    name: String,
    display_order: i64,
) -> Result<(), String> {
    state
        .db
        .update_mumble_server_group(group_id, name, display_order)
}

#[tauri::command]
fn delete_mumble_server_group(state: State<'_, AppState>, group_id: i64) -> Result<(), String> {
    state.db.delete_mumble_server_group(group_id)
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
    state.db.save_mumble_links_overlay_settings(settings)
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
    state.db.save_drawer_settings(settings)
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
        next.x = clamp_position(item.x);
        next.y = clamp_position(item.y);
        state
            .db
            .save_thumbnail_setting(payload.profile_id, item.window_title.clone(), next)?;
    }
    state.thumbnail_service.start(
        app_handle,
        state.db.clone(),
        state.window_service.clone(),
        state.dwm.clone(),
    );

    state.window_service.apply_grid_layout();
    Ok(())
}

#[tauri::command]
fn eve_profiles_list(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.eve_tools.list_profiles()
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
fn activate_window_by_pid(state: State<'_, AppState>, pid: u32) -> Result<(), String> {
    state.window_service.activate_window_by_pid(pid)
}

pub fn run() {
    if let Err(error) = instance_guard::ensure_single_instance() {
        eprintln!("single-instance check failed: {error}");
        return;
    }

    let state = match AppState::new() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("failed to initialize app state: {error}");
            return;
        }
    };

    let result = tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let state = app.state::<AppState>();
            state.thumbnail_service.start(
                app.handle().clone(),
                state.db.clone(),
                state.window_service.clone(),
                state.dwm.clone(),
            );
            if let Err(error) = setup_tray(app) {
                eprintln!("tray setup failed: {error}");
            }
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
            create_mumble_link,
            update_mumble_link,
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
            eve_profiles_list,
            eve_copy_profile,
            eve_copy_character_files,
            activate_window_by_pid
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("failed to run tauri application: {error}");
    }
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let profiles = state.db.get_profiles().unwrap_or_default();

    let mut menu_builder = MenuBuilder::new(app);
    let show_item = MenuItemBuilder::new("Show")
        .id("tray.show")
        .build(app)?;
    menu_builder = menu_builder.item(&show_item);

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

    let exit_item = MenuItemBuilder::new("Exit")
        .id("tray.exit")
        .build(app)?;
    let menu = menu_builder.item(&exit_item).build()?;

    let app_handle = app.handle().clone();
    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| {
            let id = event.id.as_ref();
            if id == "tray.show" {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
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
        settings.retain(|setting| titles.contains(&setting.window_title));
        settings.sort_by_key(|setting| {
            titles
                .iter()
                .position(|title| title == &setting.window_title)
                .unwrap_or(usize::MAX)
        });
    } else {
        settings.sort_by(|a, b| {
            a.config
                .x
                .cmp(&b.config.x)
                .then_with(|| a.window_title.cmp(&b.window_title))
        });
    }

    if payload.only_affect_active_thumbnails {
        let active_titles: Vec<String> = state
            .window_service
            .enumerate_windows()
            .into_iter()
            .map(|w| w.title)
            .collect();
        settings.retain(|setting| active_titles.contains(&setting.window_title));
    }

    let cell_height = resolve_grid_cell_height(
        payload.grid_cell_width,
        payload.grid_cell_height,
        payload.grid_cell_ratio,
    )?;
    let mut preview = Vec::new();
    for (index, setting) in settings.iter().enumerate() {
        let col = (index as i64) % payload.grid_columns;
        let row = (index as i64) / payload.grid_columns;
        preview.push(GridLayoutPreviewItem {
            window_title: setting.window_title.clone(),
            x: payload.grid_start_x + (col * payload.grid_cell_width),
            y: payload.grid_start_y + (row * cell_height),
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
