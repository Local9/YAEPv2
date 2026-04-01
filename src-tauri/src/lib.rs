mod db;
mod diag;
mod dwm;
mod thumbnail_webview_overlay;
mod monitors;
#[cfg(target_os = "windows")]
mod global_hotkeys;
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
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::db::DbService;
use crate::dwm::DwmService;
use crate::thumbnail_webview_overlay::ThumbnailOverlayStatePayload;
use crate::eve_profile_tools::EveProfileToolsService;
use crate::hotkeys::HotkeyService;
use crate::models::{
    ClientGroup, ClientGroupDetail, DrawerSettings, GridLayoutPayload, GridLayoutPreviewItem,
    HealthSnapshot, MonitorInfoDto, MumbleLink, MumbleLinksOverlaySettings, MumbleServerGroup, Profile,
    ThumbnailConfig, ThumbnailSetting,
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
    state.thumbnail_service.start(
        app_handle,
        state.db.clone(),
        state.window_service.clone(),
        state.dwm.clone(),
    );
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
    state.db.add_client_group_member(profile_id, group_id, window_title)
}

#[tauri::command]
fn remove_client_group_member(
    state: State<'_, AppState>,
    profile_id: i64,
    group_id: i64,
    window_title: String,
) -> Result<(), String> {
    state.db.remove_client_group_member(profile_id, group_id, window_title)
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
                .map(|s| {
                    (
                        s.config.x,
                        s.config.y,
                        s.config.width,
                        s.config.height,
                    )
                })
        })
        .collect();

    state.thumbnail_service.stop();
    state.db.reorder_client_group_members(
        profile_id,
        group_id,
        window_titles_in_order.clone(),
    )?;

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
            .save_thumbnail_setting(profile_id, new_title.clone(), next)?;
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
        .and_then(|title| {
            members
                .iter()
                .position(|m| m.trim() == title.trim())
        })
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

pub(crate) fn open_mumble_link_internal(state: &AppState, link_id: i64) -> Result<(), String> {
    let links = state.db.get_mumble_links()?;
    let link = links
        .into_iter()
        .find(|l| l.id == link_id)
        .ok_or_else(|| "Mumble link not found".to_string())?;
    opener::open(&link.url).map_err(|e| e.to_string())?;
    Ok(())
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
        .create_mumble_link(name, url, display_order, normalized_hotkey)?;
    refresh_global_hotkeys();
    Ok(())
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
        .update_mumble_link(link_id, name, url, display_order, normalized_hotkey)?;
    refresh_global_hotkeys();
    Ok(())
}

#[tauri::command]
fn set_mumble_link_selected(
    state: State<'_, AppState>,
    link_id: i64,
    is_selected: bool,
) -> Result<(), String> {
    state.db.set_mumble_link_selected(link_id, is_selected)?;
    refresh_global_hotkeys();
    Ok(())
}

#[tauri::command]
fn delete_mumble_link(state: State<'_, AppState>, link_id: i64) -> Result<(), String> {
    state.db.delete_mumble_link(link_id)?;
    refresh_global_hotkeys();
    Ok(())
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
            .save_thumbnail_setting(payload.profile_id, item.window_title.clone(), next)?;
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
fn eve_fetch_character_name(state: State<'_, AppState>, character_id: u64) -> Result<String, String> {
    state.eve_tools.fetch_character_name(character_id)
}

#[tauri::command]
fn activate_window_by_pid(state: State<'_, AppState>, pid: u32) -> Result<(), String> {
    state.window_service.activate_window_by_pid(pid)
}

#[tauri::command]
fn get_thumbnail_overlay_state(
    state: State<'_, AppState>,
    overlay_id: String,
) -> Option<ThumbnailOverlayStatePayload> {
    state.dwm.snapshot_thumbnail_overlay_state(&overlay_id)
}

pub fn run() {
    diag::install_panic_hook();
    diag::trace("boot", "run() entered");
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
        .manage(state)
        .setup(|app| {
            diag::trace("boot", "tauri setup callback start");
            let db = {
                let state = app.state::<AppState>();
                diag::trace("boot", "thumbnail_service.start begin");
                state.thumbnail_service.start(
                    app.handle().clone(),
                    state.db.clone(),
                    state.window_service.clone(),
                    state.dwm.clone(),
                );
                diag::trace("boot", "thumbnail_service.start returned");
                state.db.clone()
            };
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
            if let Ok(hidden) = db.get_app_setting("StartHidden".to_string()) {
                if hidden.is_some_and(|v| v.eq_ignore_ascii_case("true")) {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.hide();
                    }
                }
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
            eve_fetch_character_name,
            activate_window_by_pid,
            get_thumbnail_overlay_state
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("failed to run tauri application: {error}");
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
    let mut tray_builder = TrayIconBuilder::new().menu(&menu).tooltip(
        app.package_info().name.clone(),
    );
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
    let (ox, oy) = payload
        .selected_monitor_index
        .and_then(monitors::work_area_offset)
        .unwrap_or((0, 0));

    let (start_x, start_y) = if let Some(ref anchor_title) = payload.grid_anchor_window_title {
        let idx = settings
            .iter()
            .position(|s| s.window_title == *anchor_title)
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
        (
            ax.saturating_sub(ox as i64),
            ay.saturating_sub(oy as i64),
        )
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
