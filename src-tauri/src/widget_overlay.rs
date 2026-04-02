//! Fullscreen transparent widget overlay (Discord-style): Svelte hosts draggable widgets;
//! Windows uses a cursor poll loop to toggle `set_ignore_cursor_events` so only widget rects capture input.

use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once};
use std::time::Duration;
use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

use crate::db::DbService;
use crate::monitors;
use crate::models::{
    default_browser_quick_links, BrowserQuickLink, WidgetOverlayLayout, WidgetOverlaySettings,
};

pub const WIDGET_OVERLAY_LABEL: &str = "widget-overlay";

static HIT_REGIONS: Mutex<Vec<(i32, i32, i32, i32)>> = Mutex::new(Vec::new());
static DRAGGING: AtomicBool = AtomicBool::new(false);
static CURSOR_POLL_STARTED: Once = Once::new();

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetOverlayHitRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

fn widget_overlay_url() -> Result<WebviewUrl, String> {
    #[cfg(debug_assertions)]
    let s = "http://localhost:5173/widget-overlay";
    #[cfg(not(debug_assertions))]
    let s = "https://tauri.localhost/widget-overlay";
    Ok(WebviewUrl::External(
        s.parse::<url::Url>().map_err(|e| e.to_string())?,
    ))
}

pub fn load_settings(db: &DbService) -> Result<WidgetOverlaySettings, String> {
    let enabled = db
        .get_app_setting("WidgetOverlayEnabled".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let visible = db
        .get_app_setting("WidgetOverlayVisible".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    let monitor_index = db
        .get_app_setting("WidgetOverlayMonitorIndex".to_string())?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let show_browser_widget = db
        .get_app_setting("WidgetOverlayShowBrowser".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    let show_fleet_motd_widget = db
        .get_app_setting("WidgetOverlayShowFleetMotd".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    let show_intel_feed_widget = db
        .get_app_setting("WidgetOverlayShowIntelFeed".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    let widgets_suppressed = db
        .get_app_setting("WidgetOverlayWidgetsSuppressed".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let browser_always_displayed = db
        .get_app_setting("WidgetOverlayBrowserAlwaysDisplayed".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let fleet_motd_always_displayed = db
        .get_app_setting("WidgetOverlayFleetMotdAlwaysDisplayed".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let intel_feed_always_displayed = db
        .get_app_setting("WidgetOverlayIntelFeedAlwaysDisplayed".to_string())?
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let toggle_hotkey = db
        .get_app_setting("WidgetOverlayToggleHotkey".to_string())?
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let browser_quick_links: Vec<BrowserQuickLink> = match db
        .get_app_setting("WidgetOverlayBrowserLinksJson".to_string())?
    {
        Some(raw) if !raw.trim().is_empty() => {
            serde_json::from_str(&raw).unwrap_or_else(|_| default_browser_quick_links())
        }
        _ => default_browser_quick_links(),
    };
    let browser_default_url: Option<String> = db
        .get_app_setting("WidgetOverlayBrowserDefaultUrl".to_string())?
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let layout: WidgetOverlayLayout = db
        .get_app_setting("WidgetOverlayLayoutJson".to_string())?
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    Ok(WidgetOverlaySettings {
        enabled,
        visible,
        monitor_index,
        show_browser_widget,
        show_fleet_motd_widget,
        show_intel_feed_widget,
        widgets_suppressed,
        browser_always_displayed,
        fleet_motd_always_displayed,
        intel_feed_always_displayed,
        toggle_hotkey,
        browser_quick_links,
        browser_default_url,
        layout,
    })
}

pub fn save_settings(db: &DbService, settings: &WidgetOverlaySettings) -> Result<(), String> {
    db.set_app_setting(
        "WidgetOverlayEnabled".to_string(),
        if settings.enabled {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayVisible".to_string(),
        if settings.visible {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayMonitorIndex".to_string(),
        settings.monitor_index.to_string(),
    )?;
    db.set_app_setting(
        "WidgetOverlayShowBrowser".to_string(),
        if settings.show_browser_widget {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayShowFleetMotd".to_string(),
        if settings.show_fleet_motd_widget {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayShowIntelFeed".to_string(),
        if settings.show_intel_feed_widget {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayWidgetsSuppressed".to_string(),
        if settings.widgets_suppressed {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayBrowserAlwaysDisplayed".to_string(),
        if settings.browser_always_displayed {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayFleetMotdAlwaysDisplayed".to_string(),
        if settings.fleet_motd_always_displayed {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayIntelFeedAlwaysDisplayed".to_string(),
        if settings.intel_feed_always_displayed {
            "true".to_string()
        } else {
            "false".to_string()
        },
    )?;
    db.set_app_setting(
        "WidgetOverlayToggleHotkey".to_string(),
        settings.toggle_hotkey.clone(),
    )?;
    persist_browser_quick_links_to_db(
        db,
        &settings.browser_quick_links,
        settings.browser_default_url.as_deref(),
    )?;
    let layout_json =
        serde_json::to_string(&settings.layout).map_err(|e| e.to_string())?;
    if layout_json.len() > 8192 {
        return Err("Widget overlay layout is too large".to_string());
    }
    db.set_app_setting("WidgetOverlayLayoutJson".to_string(), layout_json)?;
    Ok(())
}

/// Persists widget positions only, merging into the latest settings from SQLite so monitor / visibility
/// changes from the main window are not overwritten by a stale overlay webview snapshot.
pub fn save_layout_only(db: &DbService, layout: &WidgetOverlayLayout) -> Result<(), String> {
    let mut settings = load_settings(db)?;
    settings.layout = layout.clone();
    save_settings(db, &settings)
}

fn validate_browser_quick_links(links: &[BrowserQuickLink]) -> Result<(), String> {
    if links.is_empty() {
        return Err("At least one browser quick link is required".to_string());
    }
    if links.len() > 24 {
        return Err("Too many browser quick links (max 24)".to_string());
    }
    let mut seen_ids = std::collections::HashSet::<String>::new();
    let mut seen_urls = std::collections::HashSet::<String>::new();
    for link in links {
        let id = link.id.trim();
        let title = link.title.trim();
        let url = link.url.trim();
        if id.is_empty() || id.len() > 64 {
            return Err("Each quick link needs a non-empty id (max 64 characters)".to_string());
        }
        if !seen_ids.insert(id.to_string()) {
            return Err("Duplicate quick link id".to_string());
        }
        if title.is_empty() || title.len() > 128 {
            return Err("Each quick link needs a title (max 128 characters)".to_string());
        }
        if url.len() > 2048 {
            return Err("Quick link URL is too long".to_string());
        }
        let lower = url.to_ascii_lowercase();
        if !lower.starts_with("https://") && !lower.starts_with("http://") {
            return Err("Quick link URLs must start with http:// or https://".to_string());
        }
        if !seen_urls.insert(url.to_string()) {
            return Err("Duplicate quick link URL".to_string());
        }
    }
    Ok(())
}

fn persist_browser_quick_links_to_db(
    db: &DbService,
    links: &[BrowserQuickLink],
    default_url: Option<&str>,
) -> Result<(), String> {
    validate_browser_quick_links(links)?;
    let def = default_url.map(str::trim).filter(|s| !s.is_empty());
    if let Some(u) = def {
        if !links.iter().any(|l| l.url.trim() == u) {
            return Err("Default URL must match one of the quick link URLs".to_string());
        }
    }
    let json = serde_json::to_string(links).map_err(|e| e.to_string())?;
    if json.len() > 24_000 {
        return Err("Browser quick links JSON is too large".to_string());
    }
    db.set_app_setting("WidgetOverlayBrowserLinksJson".to_string(), json)?;
    db.set_app_setting(
        "WidgetOverlayBrowserDefaultUrl".to_string(),
        def.unwrap_or("").to_string(),
    )?;
    Ok(())
}

pub fn save_browser_quick_links_only(
    db: &DbService,
    links: &[BrowserQuickLink],
    default_url: Option<&str>,
) -> Result<(), String> {
    persist_browser_quick_links_to_db(db, links, default_url)
}

fn monitor_rect_for_index(monitor_index: i64) -> Option<(i32, i32, u32, u32)> {
    let list = monitors::list_monitors();
    if list.is_empty() {
        return None;
    }
    let idx = usize::try_from(monitor_index).unwrap_or(0).min(list.len().saturating_sub(1));
    let m = &list[idx];
    let w = (m.right - m.left).max(1) as u32;
    let h = (m.bottom - m.top).max(1) as u32;
    Some((m.left, m.top, w, h))
}

pub fn ensure_widget_overlay_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(WIDGET_OVERLAY_LABEL).is_some() {
        return Ok(());
    }
    let url = widget_overlay_url()?;
    let win = WebviewWindowBuilder::new(app, WIDGET_OVERLAY_LABEL, url)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(false)
        .resizable(false)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = win.set_title("YAEP widget overlay");
    let _ = win.set_background_color(Some(Color(0, 0, 0, 0)));
    #[cfg(target_os = "windows")]
    {
        let _ = win.set_ignore_cursor_events(true);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = win.set_ignore_cursor_events(false);
    }
    Ok(())
}

pub fn apply_window_geometry(app: &AppHandle, settings: &WidgetOverlaySettings) -> Result<(), String> {
    let Some(win) = app.get_webview_window(WIDGET_OVERLAY_LABEL) else {
        return Ok(());
    };
    let Some((x, y, w, h)) = monitor_rect_for_index(settings.monitor_index) else {
        return Err("No displays available".to_string());
    };
    let _ = win.set_position(PhysicalPosition::new(x, y));
    let _ = win.set_size(PhysicalSize::new(w, h));
    Ok(())
}

pub fn sync_widget_overlay_from_db(app: &AppHandle, db: &DbService) {
    let settings = match load_settings(db) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("widget overlay: load settings: {e}");
            return;
        }
    };
    if !settings.enabled {
        if let Some(w) = app.get_webview_window(WIDGET_OVERLAY_LABEL) {
            let _ = w.hide();
        }
        return;
    }
    if !settings.visible {
        if let Some(w) = app.get_webview_window(WIDGET_OVERLAY_LABEL) {
            let _ = w.hide();
        }
        return;
    }
    if let Err(e) = ensure_widget_overlay_window(app) {
        eprintln!("widget overlay: ensure window: {e}");
        return;
    }
    if let Err(e) = apply_window_geometry(app, &settings) {
        eprintln!("widget overlay: geometry: {e}");
        return;
    }
    if let Some(w) = app.get_webview_window(WIDGET_OVERLAY_LABEL) {
        let _ = w.show();
        #[cfg(target_os = "windows")]
        {
            let _ = w.set_ignore_cursor_events(true);
        }
    }
}

#[cfg(target_os = "windows")]
fn point_in_regions(px: i32, py: i32, regions: &[(i32, i32, i32, i32)]) -> bool {
    for (x, y, w, h) in regions {
        if *w <= 0 || *h <= 0 {
            continue;
        }
        if px >= *x && px < x + w && py >= *y && py < y + h {
            return true;
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn poll_cursor_ignore(app: &AppHandle) {
    let Some(win) = app.get_webview_window(WIDGET_OVERLAY_LABEL) else {
        return;
    };
    if !win.is_visible().unwrap_or(false) {
        return;
    }
    if DRAGGING.load(Ordering::Relaxed) {
        let _ = win.set_ignore_cursor_events(false);
        return;
    }
    let Ok(hwnd) = win.hwnd() else {
        return;
    };
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::ScreenToClient;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut pt = POINT::default();
    unsafe {
        if GetCursorPos(&mut pt).is_err() {
            return;
        }
        if !ScreenToClient(hwnd, &mut pt).as_bool() {
            return;
        }
    }
    let regions = HIT_REGIONS.lock().unwrap();
    let hit = point_in_regions(pt.x, pt.y, &regions);
    let _ = win.set_ignore_cursor_events(!hit);
}

#[cfg(target_os = "windows")]
fn cursor_poll_loop(app: AppHandle) {
    loop {
        std::thread::sleep(Duration::from_millis(16));
        poll_cursor_ignore(&app);
    }
}

pub fn ensure_cursor_poll(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    {
        let handle = app.clone();
        CURSOR_POLL_STARTED.call_once(|| {
            std::thread::spawn(move || cursor_poll_loop(handle));
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
    }
}

pub fn update_hit_regions(rects: Vec<WidgetOverlayHitRect>) {
    let mapped: Vec<(i32, i32, i32, i32)> = rects
        .into_iter()
        .map(|r| (r.x, r.y, r.width, r.height))
        .collect();
    if let Ok(mut g) = HIT_REGIONS.lock() {
        *g = mapped;
    }
}

pub fn set_dragging(dragging: bool) {
    DRAGGING.store(dragging, Ordering::Relaxed);
}

/// Toggles whether non-pinned widgets are hidden (`widgets_suppressed`). The overlay window stays visible.
pub fn toggle_widgets_suppressed(app: &AppHandle, db: &DbService) -> Result<bool, String> {
    let mut settings = load_settings(db)?;
    if !settings.enabled {
        return Err("Enable the widget overlay in settings first.".to_string());
    }
    settings.widgets_suppressed = !settings.widgets_suppressed;
    save_settings(db, &settings)?;
    sync_widget_overlay_from_db(app, db);
    let _ = app.emit("widget-overlay-settings-changed", ());
    Ok(settings.widgets_suppressed)
}
