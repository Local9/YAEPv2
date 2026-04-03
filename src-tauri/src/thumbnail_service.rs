use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

use serde::Serialize;
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::db::DbService;
use crate::diag;
use crate::eve_preview_windows::try_list_preview_eve_windows;
use crate::dwm::DwmService;
use crate::windows::{WindowService, WindowSnapshot};

const MONITOR_INTERVAL_MS: u64 = 2000;
const FOCUS_INTERVAL_MS: u64 = 100;
const NEW_THUMBNAIL_REGISTRATION_BREATHING_MS: u64 = 140;
const OVERLAY_CREATION_PER_CYCLE: usize = 1;

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
pub struct FocusEvent {
    pid: Option<u32>,
    window_title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeThumbnailSnapshot {
    pub pid: u32,
    pub window_title: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeThumbnailStateSnapshot {
    pub thumbnails: Vec<RuntimeThumbnailSnapshot>,
    pub focused: FocusEvent,
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

    pub fn snapshot_state(&self) -> RuntimeThumbnailStateSnapshot {
        let Ok(state) = self.state.lock() else {
            return RuntimeThumbnailStateSnapshot {
                thumbnails: Vec::new(),
                focused: FocusEvent {
                    pid: None,
                    window_title: None,
                },
            };
        };

        let mut thumbnails: Vec<RuntimeThumbnailSnapshot> = state
            .thumbnails_by_pid
            .iter()
            .map(|(pid, thumb)| RuntimeThumbnailSnapshot {
                pid: *pid,
                window_title: thumb.title.clone(),
            })
            .collect();
        thumbnails.sort_by_key(|t| t.pid);

        let focused = match state.focused_pid {
            Some(pid) => FocusEvent {
                pid: Some(pid),
                window_title: state.thumbnails_by_pid.get(&pid).map(|thumb| thumb.title.clone()),
            },
            None => FocusEvent {
                pid: None,
                window_title: None,
            },
        };

        RuntimeThumbnailStateSnapshot {
            thumbnails,
            focused,
        }
    }
}

/// Runs [`DwmService::sync_thumbnail_graph`] when [`DwmService::request_thumbnail_layout_sync`] was
/// set, even if this monitor cycle returns early (no active profile / no processes).
struct DeferredThumbnailLayoutSync {
    dwm: Arc<DwmService>,
}

impl Drop for DeferredThumbnailLayoutSync {
    fn drop(&mut self) {
        if self.dwm.take_pending_thumbnail_layout_sync() {
            self.dwm.sync_thumbnail_graph();
        }
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
    let _deferred_layout_sync = DeferredThumbnailLayoutSync {
        dwm: dwm.clone(),
    };

    let Some(filtered) = try_list_preview_eve_windows(db, windows, sys) else {
        return;
    };

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

    // Snapshot under lock, then release before DWM calls. DWM uses `run_on_main` and blocks until
    // the GUI thread runs; holding `runtime` here deadlocks if `ThumbnailService::stop()` runs on
    // that thread (e.g. settings import after the file dialog).
    let (previous_by_pid, previous_focused) = {
        let state = runtime.lock().expect("thumbnail runtime lock poisoned");
        (
            state.thumbnails_by_pid.clone(),
            state.focused_pid,
        )
    };

    let mut new_pids: Vec<u32> = new_by_pid.keys().copied().collect();
    new_pids.sort_unstable();
    for pid in new_pids {
        let Some(thumb) = new_by_pid.get(&pid) else {
            continue;
        };
        if !previous_by_pid.contains_key(&pid) {
            let _ = app_handle.emit(
                "thumbnailAdded",
                ThumbnailEvent {
                    pid,
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
            dwm.register_runtime_thumbnail(pid, thumb.hwnd, &thumb.title);
            thread::sleep(Duration::from_millis(
                NEW_THUMBNAIL_REGISTRATION_BREATHING_MS,
            ));
            continue;
        }

        if let Some(previous) = previous_by_pid.get(&pid) {
            let title_changed = previous.title != thumb.title;
            let hwnd_changed = previous.hwnd != thumb.hwnd;
            if title_changed || hwnd_changed {
                let _ = app_handle.emit(
                    "thumbnailUpdated",
                    ThumbnailEvent {
                        pid,
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
                dwm.register_runtime_thumbnail(pid, thumb.hwnd, &thumb.title);
            }
        }
    }

    let removed: Vec<u32> = previous_by_pid
        .keys()
        .copied()
        .filter(|pid| !new_by_pid.contains_key(pid))
        .collect();
    let mut cleared_focus = false;
    for pid in removed {
        if let Some(old) = previous_by_pid.get(&pid) {
            let _ = app_handle.emit(
                "thumbnailRemoved",
                ThumbnailEvent {
                    pid,
                    window_title: old.title.clone(),
                },
            );
        }
        dwm.unregister_runtime_thumbnail(pid);
        if previous_focused == Some(pid) {
            dwm.set_focused_thumbnail(None);
            let _ = app_handle.emit(
                "focusChanged",
                FocusEvent {
                    pid: None,
                    window_title: None,
                },
            );
            cleared_focus = true;
        }
    }

    let mut state = runtime.lock().expect("thumbnail runtime lock poisoned");
    state.thumbnails_by_pid = new_by_pid;
    if cleared_focus {
        state.focused_pid = None;
    }
    drop(state);
    dwm.ensure_missing_runtime_overlays(OVERLAY_CREATION_PER_CYCLE);
}

fn refresh_focus_state(
    runtime: &Arc<Mutex<RuntimeState>>,
    app_handle: &AppHandle,
    windows: &Arc<WindowService>,
    dwm: &Arc<DwmService>,
) {
    let foreground_pid = windows.foreground_window_pid();
    let Some(pid) = foreground_pid else {
        return;
    };
    let (window_title, previous_focused) = {
        let state = runtime.lock().expect("thumbnail runtime lock poisoned");
        let Some(window) = state.thumbnails_by_pid.get(&pid) else {
            // QoL: keep last focused state when user focuses non-thumbnail windows.
            return;
        };
        if state.focused_pid == Some(pid) {
            return;
        }
        (window.title.clone(), state.focused_pid)
    };

    dwm.set_focused_thumbnail(Some(pid));

    let mut state = runtime.lock().expect("thumbnail runtime lock poisoned");
    if state.thumbnails_by_pid.get(&pid).is_none() {
        drop(state);
        dwm.set_focused_thumbnail(previous_focused);
        return;
    }
    if state.focused_pid == Some(pid) {
        return;
    }
    state.focused_pid = Some(pid);
    drop(state);

    let _ = app_handle.emit(
        "focusChanged",
        FocusEvent {
            pid: Some(pid),
            window_title: Some(window_title),
        },
    );
}

