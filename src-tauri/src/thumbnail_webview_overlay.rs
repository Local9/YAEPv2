//! **ThumbnailOverlayWindow** — Tauri `WebviewWindow` for Svelte `/thumbnail-overlay` (border, title).
//!
//! Two sibling windows, stacked **DWM thumbnail (bottom) / webview (top)**:
//! - On Windows the overlay uses **`owner_raw(thumbnail_hwnd)`** so Win32 keeps the owned window
//!   above the thumbnail window.
//! - **[`sync_overlay_bounds_win`]** uses one **`SetWindowPos`** (position + size + insert-after) so
//!   layout and Z-order apply atomically (avoids Tauri async `set_position` undoing Z-order).

use std::ffi::c_void;

use serde::Serialize;
use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, SetWindowPos, SWP_NOACTIVATE};

use crate::models::ThumbnailConfig;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlayStatePayload {
    pub overlay_id: String,
    pub pid: u32,
    pub focused: bool,
    pub focus_border_color: String,
    pub focus_border_thickness: i64,
    pub show_title_overlay: bool,
    pub title: String,
}

pub fn new_overlay_id() -> String {
    Uuid::new_v4().to_string()
}

/// Tauri window label (alphanumeric only; UUID hyphens stripped).
pub fn overlay_window_label(overlay_id: &str) -> String {
    format!("thumb{}", overlay_id.replace('-', ""))
}

fn overlay_entry_url(_app: &AppHandle, overlay_id: &str, pid: u32) -> Result<WebviewUrl, String> {
    #[cfg(debug_assertions)]
    let s = format!(
        "http://localhost:5173/thumbnail-overlay?overlayId={}&pid={}",
        overlay_id, pid
    );
    #[cfg(not(debug_assertions))]
    let s = format!(
        "https://tauri.localhost/thumbnail-overlay?overlayId={}&pid={}",
        overlay_id, pid
    );
    Ok(WebviewUrl::External(
        s.parse::<url::Url>().map_err(|e| e.to_string())?,
    ))
}

/// Opens the overlay as a **separate** top-level window, **owned** by the native thumbnail HWND on Windows
/// so it stays above the DWM surface. `thumbnail_window_hwnd` must be the window that hosts `DwmRegisterThumbnail`.
pub fn open_thumbnail_overlay_window(
    app: &AppHandle,
    overlay_id: &str,
    pid: u32,
    thumbnail_window_hwnd: isize,
) -> Result<String, String> {
    let label = overlay_window_label(overlay_id);
    if app.get_webview_window(&label).is_some() {
        return Ok(label);
    }
    let url = overlay_entry_url(app, overlay_id, pid)?;
    #[cfg(target_os = "windows")]
    let win = {
        let mut b = WebviewWindowBuilder::new(app, &label, url)
            .decorations(false)
            .transparent(true)
            .background_color(Color(0, 0, 0, 0))
            .shadow(false)
            .skip_taskbar(true)
            .visible(false)
            .resizable(false);
        if thumbnail_window_hwnd != 0 {
            b = b.owner_raw(HWND(thumbnail_window_hwnd as *mut c_void));
        } else {
            b = b.always_on_top(true);
        }
        b.build().map_err(|e| e.to_string())?
    };
    #[cfg(not(target_os = "windows"))]
    let win = WebviewWindowBuilder::new(app, &label, url)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .resizable(false)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = win.set_title("YAEP thumbnail overlay");
    let _ = win.set_background_color(Some(Color(0, 0, 0, 0)));
    let _ = win.set_ignore_cursor_events(true);
    Ok(label)
}

pub fn close_thumbnail_overlay_window(app: &AppHandle, label: &str) {
    if label.is_empty() {
        return;
    }
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.close();
    }
}

pub fn show_overlay_window(app: &AppHandle, label: &str) {
    if label.is_empty() {
        return;
    }
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.show();
        let _ = w.set_ignore_cursor_events(true);
    }
}

pub fn set_overlay_window_title(app: &AppHandle, label: &str, title: &str) {
    if label.is_empty() {
        return;
    }
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.set_title(title);
    }
}

/// Match overlay to the thumbnail window’s screen rect and stack **directly above** it (one `SetWindowPos`).
pub fn sync_overlay_bounds_win(
    app: &AppHandle,
    thumbnail_window_hwnd: isize,
    overlay_label: &str,
) {
    if thumbnail_window_hwnd == 0 || overlay_label.is_empty() {
        return;
    }
    let Some(win) = app.get_webview_window(overlay_label) else {
        return;
    };
    let mut rect = RECT::default();
    if unsafe {
        GetWindowRect(
            HWND(thumbnail_window_hwnd as *mut c_void),
            &mut rect,
        )
        .is_err()
    } {
        return;
    }
    let w = (rect.right - rect.left).max(1);
    let h = (rect.bottom - rect.top).max(1);
    #[cfg(target_os = "windows")]
    {
        let Ok(overlay_hwnd) = win.hwnd() else {
            return;
        };
        unsafe {
            let _ = SetWindowPos(
                overlay_hwnd,
                Some(HWND(thumbnail_window_hwnd as *mut c_void)),
                rect.left,
                rect.top,
                w,
                h,
                SWP_NOACTIVATE,
            );
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
            rect.left, rect.top,
        )));
        let _ = win.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
            w as u32,
            h as u32,
        )));
    }
    let _ = win.set_ignore_cursor_events(true);
}

pub fn overlay_state_payload(
    overlay_id: &str,
    pid: u32,
    focused: bool,
    config: &ThumbnailConfig,
    window_title: &str,
) -> ThumbnailOverlayStatePayload {
    let display_title = window_title
        .strip_prefix("EVE - ")
        .unwrap_or(window_title)
        .to_string();
    ThumbnailOverlayStatePayload {
        overlay_id: overlay_id.to_string(),
        pid,
        focused,
        focus_border_color: config.focus_border_color.clone(),
        focus_border_thickness: config.focus_border_thickness,
        show_title_overlay: config.show_title_overlay,
        title: display_title,
    }
}

pub fn emit_overlay_state(
    app: &AppHandle,
    overlay_label: &str,
    overlay_id: &str,
    pid: u32,
    focused: bool,
    config: &ThumbnailConfig,
    window_title: &str,
) {
    if overlay_label.is_empty() {
        return;
    }
    let payload = overlay_state_payload(overlay_id, pid, focused, config, window_title);
    let _ = app.emit_to(overlay_label, "thumbnail-overlay:state", payload);
}
