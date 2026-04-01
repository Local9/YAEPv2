use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::db::DbService;
use crate::diag;
use crate::dwm::DwmService;
use crate::windows::{WindowService, WindowSnapshot};

const MONITOR_INTERVAL_MS: u64 = 2000;
const FOCUS_INTERVAL_MS: u64 = 100;
const BASE_EVE_TITLE: &str = "EVE";
const CHARACTER_TITLE_PREFIX: &str = "EVE - ";

#[derive(Debug, Clone)]
struct RuntimeThumbnail {
    hwnd: isize,
    title: String,
}

#[derive(Default)]
struct RuntimeState {
    running: bool,
    stop_flag: Option<Arc<AtomicBool>>,
    thumbnails_by_pid: HashMap<u32, RuntimeThumbnail>,
    focused_pid: Option<u32>,
}

#[derive(Default)]
pub struct ThumbnailService {
    state: Arc<Mutex<RuntimeState>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ThumbnailEvent {
    pid: u32,
    window_title: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FocusEvent {
    pid: Option<u32>,
    window_title: Option<String>,
}

impl ThumbnailService {
    pub fn start(
        &self,
        app_handle: AppHandle,
        db: Arc<DbService>,
        windows: Arc<WindowService>,
        dwm: Arc<DwmService>,
    ) {
        let stop_flag = {
            let mut state = self.state.lock().expect("thumbnail runtime lock poisoned");
            if state.running {
                return;
            }
            let stop_flag = Arc::new(AtomicBool::new(false));
            state.running = true;
            state.stop_flag = Some(stop_flag.clone());
            state.focused_pid = None;
            stop_flag
        };

        diag::trace("thumbnail", "start(): set_app_handle + set_db");
        dwm.set_app_handle(app_handle.clone());
        dwm.set_db(db.clone());
        diag::trace("thumbnail", "start(): spawning monitor + focus tasks");

        let monitor_state = self.state.clone();
        let monitor_stop = stop_flag.clone();
        let monitor_app = app_handle.clone();
        let monitor_db = db.clone();
        let monitor_windows = windows.clone();
        let monitor_dwm = dwm.clone();
        tauri::async_runtime::spawn(async move {
            let mut sys = System::new_all();
            loop {
                if monitor_stop.load(Ordering::Relaxed) {
                    break;
                }
                refresh_runtime_thumbnails(
                    &monitor_state,
                    &monitor_app,
                    &monitor_db,
                    &monitor_windows,
                    &monitor_dwm,
                    &mut sys,
                );
                sleep(Duration::from_millis(MONITOR_INTERVAL_MS)).await;
            }
        });

        let focus_state = self.state.clone();
        let focus_stop = stop_flag;
        let focus_app = app_handle;
        tauri::async_runtime::spawn(async move {
            loop {
                if focus_stop.load(Ordering::Relaxed) {
                    break;
                }
                refresh_focus_state(&focus_state, &focus_app, &windows, &dwm);
                sleep(Duration::from_millis(FOCUS_INTERVAL_MS)).await;
            }
        });
    }

    pub fn stop(&self) {
        let mut state = self.state.lock().expect("thumbnail runtime lock poisoned");
        if let Some(stop_flag) = &state.stop_flag {
            stop_flag.store(true, Ordering::Relaxed);
        }
        state.running = false;
        state.stop_flag = None;
        state.focused_pid = None;
        state.thumbnails_by_pid.clear();
    }

    /// True if the foreground HWND belongs to a PID we are currently tracking as a thumbnail client.
    pub fn is_foreground_a_runtime_thumbnail(&self, windows: &WindowService) -> bool {
        let Some(fg) = windows.foreground_window_snapshot() else {
            return false;
        };
        let Ok(state) = self.state.lock() else {
            return false;
        };
        state.thumbnails_by_pid.contains_key(&fg.pid)
    }

    /// Group member titles that currently match a tracked thumbnail, preserving group list order.
    pub fn filter_group_members_to_active_runtime(
        &self,
        group_member_titles: &[String],
    ) -> Vec<String> {
        let Ok(state) = self.state.lock() else {
            return Vec::new();
        };
        group_member_titles
            .iter()
            .filter(|m| {
                let mt = m.trim();
                state
                    .thumbnails_by_pid
                    .values()
                    .any(|th| th.title.trim() == mt)
            })
            .cloned()
            .collect()
    }

    /// Focus the tracked client whose window title matches (trimmed). Uses known PID/HWND from the runtime map.
    pub fn focus_thumbnail_client_by_title(
        &self,
        window_title: &str,
        windows: &WindowService,
    ) -> Result<(), String> {
        let t = window_title.trim();
        let state = self
            .state
            .lock()
            .map_err(|_| "thumbnail runtime lock poisoned".to_string())?;
        let Some((&pid, _thumb)) = state
            .thumbnails_by_pid
            .iter()
            .find(|(_, th)| th.title.trim() == t)
        else {
            return Err("no runtime thumbnail for that title".to_string());
        };
        drop(state);
        windows.activate_window_by_pid(pid)
    }
}

fn refresh_runtime_thumbnails(
    runtime: &Arc<Mutex<RuntimeState>>,
    app_handle: &AppHandle,
    db: &Arc<DbService>,
    windows: &Arc<WindowService>,
    dwm: &Arc<DwmService>,
    sys: &mut System,
) {
    let Some(active_profile_id) = db.active_profile_id() else {
        return;
    };

    let Ok(process_names) = db.get_processes_to_preview(active_profile_id) else {
        return;
    };
    let target_processes: Vec<String> = process_names
        .into_iter()
        .map(|p| normalize_process_name(&p))
        .collect();

    sys.refresh_processes(ProcessesToUpdate::All, true);
    let candidates = windows.enumerate_windows();
    let filtered = filter_windows(candidates, sys, &target_processes);

    let mut state = runtime.lock().expect("thumbnail runtime lock poisoned");
    let new_by_pid: HashMap<u32, RuntimeThumbnail> = filtered
        .into_iter()
        .map(|window| {
            (
                window.pid,
                RuntimeThumbnail {
                    hwnd: window.hwnd,
                    title: window.title,
                },
            )
        })
        .collect();

    for (pid, thumb) in &new_by_pid {
        if !state.thumbnails_by_pid.contains_key(pid) {
            let _ = app_handle.emit(
                "thumbnailAdded",
                ThumbnailEvent {
                    pid: *pid,
                    window_title: thumb.title.clone(),
                },
            );
            diag::trace(
                "thumbnail",
                &format!(
                    "new pid={pid} hwnd=0x{:x} title={}",
                    thumb.hwnd, thumb.title
                ),
            );
            dwm.register_runtime_thumbnail(*pid, thumb.hwnd, &thumb.title);
            continue;
        }

        if let Some(previous) = state.thumbnails_by_pid.get(pid) {
            let title_changed = previous.title != thumb.title;
            let hwnd_changed = previous.hwnd != thumb.hwnd;
            if title_changed || hwnd_changed {
                let _ = app_handle.emit(
                    "thumbnailUpdated",
                    ThumbnailEvent {
                        pid: *pid,
                        window_title: thumb.title.clone(),
                    },
                );
                diag::trace(
                    "thumbnail",
                    &format!(
                        "refresh pid={pid} hwnd/title change -> re-register hwnd=0x{:x}",
                        thumb.hwnd
                    ),
                );
                // Keep DWM linkage synchronized if source window changed.
                dwm.register_runtime_thumbnail(*pid, thumb.hwnd, &thumb.title);
            }
        }
    }

    let removed: Vec<u32> = state
        .thumbnails_by_pid
        .keys()
        .copied()
        .filter(|pid| !new_by_pid.contains_key(pid))
        .collect();
    for pid in removed {
        if let Some(old) = state.thumbnails_by_pid.get(&pid) {
            let _ = app_handle.emit(
                "thumbnailRemoved",
                ThumbnailEvent {
                    pid,
                    window_title: old.title.clone(),
                },
            );
        }
        dwm.unregister_runtime_thumbnail(pid);
        if state.focused_pid == Some(pid) {
            state.focused_pid = None;
            dwm.set_focused_thumbnail(None);
            let _ = app_handle.emit(
                "focusChanged",
                FocusEvent {
                    pid: None,
                    window_title: None,
                },
            );
        }
    }

    state.thumbnails_by_pid = new_by_pid;
}

fn refresh_focus_state(
    runtime: &Arc<Mutex<RuntimeState>>,
    app_handle: &AppHandle,
    windows: &Arc<WindowService>,
    dwm: &Arc<DwmService>,
) {
    let foreground_pid = windows.foreground_window_pid();
    let mut state = runtime.lock().expect("thumbnail runtime lock poisoned");
    let Some(pid) = foreground_pid else {
        return;
    };
    let Some(window) = state.thumbnails_by_pid.get(&pid) else {
        // QoL: keep last focused state when user focuses non-thumbnail windows.
        return;
    };
    if state.focused_pid == Some(pid) {
        return;
    }
    let window_title = window.title.clone();
    state.focused_pid = Some(pid);
    dwm.set_focused_thumbnail(Some(pid));
    let _ = app_handle.emit(
        "focusChanged",
        FocusEvent {
            pid: Some(pid),
            window_title: Some(window_title),
        },
    );
}

fn filter_windows(
    windows: Vec<WindowSnapshot>,
    sys: &System,
    target_processes: &[String],
) -> Vec<WindowSnapshot> {
    windows
        .into_iter()
        .filter(|window| {
            let pid = Pid::from_u32(window.pid);
            let Some(process) = sys.process(pid) else {
                return false;
            };
            let process_name = normalize_process_name(process.name().to_string_lossy().as_ref());
            if !target_processes.contains(&process_name) {
                return false;
            }

            if window.title.trim() == BASE_EVE_TITLE {
                return false;
            }
            window.title.starts_with(CHARACTER_TITLE_PREFIX)
        })
        .collect()
}

fn normalize_process_name(name: &str) -> String {
    name.trim().trim_end_matches(".exe").to_lowercase()
}
