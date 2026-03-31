use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};

use tauri::AppHandle;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::Graphics::Dwm::{
    DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
    DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION,
    DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetCapture, GetKeyState, ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE,
    TRACKMOUSEEVENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetCursorPos,
    GetWindowRect, IsIconic, IsWindow, LoadCursorW, RegisterClassW, SetCursor, SetForegroundWindow,
    SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow, CW_USEDEFAULT, GWLP_USERDATA,
    IDC_ARROW, SW_RESTORE, SW_SHOWNA, SWP_NOACTIVATE, SWP_NOZORDER,
    WINDOW_EX_STYLE,
    WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_MBUTTONDOWN, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_SETCURSOR, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};

use windows::core::PCWSTR;

use crate::db::DbService;
use crate::diag;
use crate::models::ThumbnailConfig;
use crate::thumbnail_webview_overlay;

static DWM_APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

fn dwm_app_handle() -> Option<AppHandle> {
    DWM_APP_HANDLE.lock().ok().and_then(|g| g.clone())
}

/// Layout of [`DWM_THUMBNAIL_PROPERTIES`] as used by `dwmapi.dll` (MSVC: padding after `opacity`).
/// The `windows` crate binds this type as `repr(C, packed(1))`, which is the wrong size/layout for the DLL.
#[repr(C)]
struct DwmThumbnailPropertiesAbi {
    dw_flags: u32,
    rc_destination: RECT,
    rc_source: RECT,
    opacity: u8,
    _pad: [u8; 3],
    f_visible: i32,
    f_source_client_area_only: i32,
}

/// `DwmUpdateThumbnailProperties` mask: destination rect (required for layout) plus
/// `DWM_TNP_OPACITY` (0x4), `DWM_TNP_VISIBLE` (0x8), `DWM_TNP_SOURCECLIENTAREAONLY` (0x10).
const DWM_THUMBNAIL_UPDATE_FLAGS: u32 = DWM_TNP_RECTDESTINATION
    | DWM_TNP_OPACITY
    | DWM_TNP_VISIBLE
    | DWM_TNP_SOURCECLIENTAREAONLY;

const MIN_THUMB_W: i32 = 192;
const MIN_THUMB_H: i32 = 108;
const MAX_THUMB_W: i32 = 960;
const MAX_THUMB_H: i32 = 540;
const POS_LOW: i64 = -10_000;
const POS_HIGH: i64 = 31_000;
const VK_CONTROL: i32 = 0x11;
const MK_CONTROL_WPARAM: u32 = 0x0008;
const WM_MOUSE_LEAVE: u32 = 0x02A3; // WM_MOUSELEAVE

// Avalonia-aligned layout (two synced top-level windows):
// - **ThumbnailWindow** — native `WS_POPUP` here; **only** DWM live preview into its client area
//   (`DwmRegisterThumbnail` / `DwmUpdateThumbnailProperties`). Input: `thumbnail_container_wnd_proc`.
// - **ThumbnailOverlayWindow** — separate Tauri `WebviewWindow` (`thumbnail_webview_overlay`), border/title
//   only; positioned and Z-ordered **above** the thumbnail window via `sync_overlay_bounds`.
//   `set_ignore_cursor_events` so clicks hit the thumbnail window.
// **DWM preview** opacity uses `ThumbnailConfig.opacity` and goes fully opaque while the cursor is
// over the thumbnail window (mouse enter/leave). The **overlay** webview never reads that opacity;
// it only shows border/title (`emit_overlay_state` does not send opacity).

#[derive(Debug, Clone)]
struct RuntimeThumbnailWindow {
    /// Native **ThumbnailWindow** HWND (DWM destination only).
    thumbnail_window_hwnd: isize,
    overlay_id: String,
    overlay_label: String,
    source_hwnd: isize,
    thumbnail: isize,
    is_focused: bool,
    window_title: String,
    config: ThumbnailConfig,
}

struct DwmInner {
    runtime: Mutex<HashMap<u32, RuntimeThumbnailWindow>>,
    app_handle: Mutex<Option<AppHandle>>,
    db: Mutex<Option<Arc<DbService>>>,
}

/// DWM thumbnail hosts must be created and messaged on the thread that runs a Win32 message loop.
#[derive(Clone)]
pub struct DwmService {
    inner: Arc<DwmInner>,
}

impl Default for DwmService {
    fn default() -> Self {
        Self {
            inner: Arc::new(DwmInner::default()),
        }
    }
}

impl Default for DwmInner {
    fn default() -> Self {
        Self {
            runtime: Mutex::new(HashMap::new()),
            app_handle: Mutex::new(None),
            db: Mutex::new(None),
        }
    }
}

/// For [`thumbnail_container_wnd_proc`] (no `&DwmInner`): DB writes on drag/wheel end.
static GLOBAL_DB: Mutex<Option<Arc<DbService>>> = Mutex::new(None);

#[derive(Clone)]
struct HostInteractionInfo {
    pid: u32,
    window_title: String,
    thumbnail: isize,
}

fn interaction_map() -> &'static Mutex<HashMap<isize, HostInteractionInfo>> {
    static M: OnceLock<Mutex<HashMap<isize, HostInteractionInfo>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
struct LayoutEntry {
    thumbnail_window_hwnd: isize,
    overlay_label: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

fn layout_snapshot() -> &'static Mutex<HashMap<u32, LayoutEntry>> {
    static M: OnceLock<Mutex<HashMap<u32, LayoutEntry>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
struct DragState {
    anchor: POINT,
    start_rects: HashMap<u32, (i32, i32, i32, i32)>,
    primary_pid: u32,
    group: bool,
}

fn drag_state() -> &'static Mutex<Option<DragState>> {
    static M: OnceLock<Mutex<Option<DragState>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(None))
}

fn hover_thumbnail_windows() -> &'static Mutex<HashSet<isize>> {
    static M: OnceLock<Mutex<HashSet<isize>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashSet::new()))
}

fn thumbnail_window_is_hovered(thumbnail_window_hwnd: isize) -> bool {
    hover_thumbnail_windows()
        .lock()
        .map(|h| h.contains(&thumbnail_window_hwnd))
        .unwrap_or(false)
}

fn set_thumbnail_window_hovered(thumbnail_window_hwnd: isize, hovered: bool) {
    let Ok(mut set) = hover_thumbnail_windows().lock() else {
        return;
    };
    if hovered {
        set.insert(thumbnail_window_hwnd);
    } else {
        set.remove(&thumbnail_window_hwnd);
    }
}

impl DwmService {
    pub fn sync_thumbnail_graph(&self) {
        let inner = self.inner.clone();
        self.run_on_main(move || sync_thumbnail_graph_locked(&inner));
    }

    pub fn set_app_handle(&self, app: AppHandle) {
        *self.inner.app_handle.lock().expect("dwm app_handle lock poisoned") = Some(app.clone());
        if let Ok(mut g) = DWM_APP_HANDLE.lock() {
            *g = Some(app);
        }
    }

    pub fn set_db(&self, db: Arc<DbService>) {
        *self.inner.db.lock().expect("dwm db lock poisoned") = Some(db.clone());
        *GLOBAL_DB.lock().expect("global db lock poisoned") = Some(db);
    }

    fn run_on_main(&self, f: impl FnOnce() + Send + 'static) {
        let app = self
            .inner
            .app_handle
            .lock()
            .expect("dwm app_handle lock poisoned")
            .clone();
        let Some(app) = app else {
            eprintln!("YAEP: DwmService app handle not set; skipping DWM operation");
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        if let Err(e) = app.run_on_main_thread(move || {
            f();
            let _ = tx.send(());
        }) {
            eprintln!("YAEP: run_on_main_thread failed: {e}");
            return;
        }
        let _ = rx.recv();
    }

    pub fn register_runtime_thumbnail(&self, pid: u32, source_hwnd: isize, title: &str) {
        if source_hwnd == 0 {
            diag::trace("dwm", &format!("register_runtime_thumbnail pid={pid} skipped: source_hwnd=0"));
            return;
        }
        diag::trace(
            "dwm",
            &format!(
                "register_runtime_thumbnail pid={pid} source_hwnd=0x{source_hwnd:x} enqueue main thread"
            ),
        );
        let title = title.to_string();
        let inner = self.inner.clone();
        self.run_on_main(move || {
            diag::trace(
                "dwm",
                &format!("register_runtime_thumbnail pid={pid} on main thread"),
            );
            register_runtime_thumbnail_locked(&inner, pid, source_hwnd, title);
            diag::trace(
                "dwm",
                &format!("register_runtime_thumbnail pid={pid} main thread done"),
            );
        });
    }

    pub fn unregister_runtime_thumbnail(&self, pid: u32) {
        let inner = self.inner.clone();
        self.run_on_main(move || unregister_runtime_thumbnail_locked(&inner, pid));
    }

    pub fn set_focused_thumbnail(&self, focused_pid: Option<u32>) {
        let inner = self.inner.clone();
        self.run_on_main(move || set_focused_thumbnail_locked(&inner, focused_pid));
    }

    /// Current overlay payload for a thumbnail (for webview hydration if the first `emit` was missed).
    pub fn snapshot_thumbnail_overlay_state(
        &self,
        overlay_id: &str,
    ) -> Option<crate::thumbnail_webview_overlay::ThumbnailOverlayStatePayload> {
        let inner = self.inner.clone();
        let overlay_id = overlay_id.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        let app = self
            .inner
            .app_handle
            .lock()
            .ok()
            .and_then(|g| g.clone())?;
        if app
            .run_on_main_thread(move || {
                let out = inner
                    .runtime
                    .lock()
                    .ok()
                    .and_then(|s| {
                        s.iter().find(|(_, e)| e.overlay_id == overlay_id).map(
                            |(pid, e)| {
                                thumbnail_webview_overlay::overlay_state_payload(
                                    overlay_id.as_str(),
                                    *pid,
                                    e.is_focused,
                                    &e.config,
                                    &e.window_title,
                                )
                            },
                        )
                    });
                let _ = tx.send(out);
            })
            .is_err()
        {
            return None;
        }
        rx.recv().ok().flatten()
    }
}

fn sync_thumbnail_graph_locked(inner: &Arc<DwmInner>) {
    let db_guard = inner.db.lock().expect("dwm db lock poisoned").clone();
    let Some(db) = db_guard else {
        return;
    };
    let Some(profile_id) = db.active_profile_id() else {
        return;
    };
    let mut state = inner.runtime.lock().expect("dwm runtime lock poisoned");
    for (pid, entry) in state.iter_mut() {
        let Ok(config) = db.resolve_thumbnail_config(profile_id, &entry.window_title) else {
            continue;
        };
        entry.config = config.clone();
        let hovered = thumbnail_window_is_hovered(entry.thumbnail_window_hwnd);
        apply_thumbnail_container_geometry(
            entry.thumbnail_window_hwnd,
            entry.overlay_label.as_str(),
            &config,
            entry.thumbnail,
            hovered,
        );
        layout_snapshot_update(
            *pid,
            entry.thumbnail_window_hwnd,
            entry.overlay_label.clone(),
            config.x,
            config.y,
            config.width,
            config.height,
        );
        if let Some(app) = dwm_app_handle() {
            thumbnail_webview_overlay::emit_overlay_state(
                &app,
                &entry.overlay_label,
                &entry.overlay_id,
                *pid,
                entry.is_focused,
                &config,
                &entry.window_title,
            );
        }
    }
}

fn layout_snapshot_update(
    pid: u32,
    thumbnail_window_hwnd: isize,
    overlay_label: String,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
) {
    let (cw, ch) = clamp_dims_i64(w, h);
    let mut m = layout_snapshot().lock().expect("layout snapshot lock poisoned");
    m.insert(
        pid,
        LayoutEntry {
            thumbnail_window_hwnd,
            overlay_label,
            x: clamp_pos_i64(x),
            y: clamp_pos_i64(y),
            w: cw,
            h: ch,
        },
    );
}

fn layout_snapshot_remove(pid: u32) {
    let mut m = layout_snapshot().lock().expect("layout snapshot lock poisoned");
    m.remove(&pid);
}

fn clamp_pos_i64(v: i64) -> i32 {
    v.clamp(POS_LOW, POS_HIGH) as i32
}

fn clamp_dims_i64(w: i64, h: i64) -> (i32, i32) {
    let mut cw = w.clamp(MIN_THUMB_W as i64, MAX_THUMB_W as i64) as i32;
    let mut ch = h.clamp(MIN_THUMB_H as i64, MAX_THUMB_H as i64) as i32;
    cw = cw.clamp(MIN_THUMB_W, MAX_THUMB_W);
    ch = ch.clamp(MIN_THUMB_H, MAX_THUMB_H);
    (cw, ch)
}

fn register_runtime_thumbnail_locked(
    inner: &Arc<DwmInner>,
    pid: u32,
    source_hwnd: isize,
    title: String,
) {
    let db_guard = inner.db.lock().expect("dwm db lock poisoned").clone();
    let Some(db) = db_guard else {
        eprintln!("YAEP: DwmService DB not set; cannot place thumbnail");
        return;
    };
    let Some(profile_id) = db.active_profile_id() else {
        diag::trace("dwm", &format!("pid={pid} skip: no active profile"));
        return;
    };
    let Ok(mut config) = db.resolve_thumbnail_config(profile_id, &title) else {
        diag::trace("dwm", &format!("pid={pid} skip: resolve_thumbnail_config failed"));
        return;
    };
    let (cw, ch) = clamp_dims_i64(config.width, config.height);
    config.width = cw as i64;
    config.height = ch as i64;
    config.x = config.x.clamp(POS_LOW, POS_HIGH);
    config.y = config.y.clamp(POS_LOW, POS_HIGH);

    let class_name = register_thumbnail_window_class();
    let mut state = inner.runtime.lock().expect("dwm runtime lock poisoned");

    let app_for_overlay = dwm_app_handle();
    let entry = state.entry(pid).or_insert_with(|| {
        diag::trace(
            "dwm",
            &format!("pid={pid} new entry: native ThumbnailWindow + ThumbnailOverlayWindow (Tauri)"),
        );
        let thumbnail_window_hwnd = create_thumbnail_window(
            class_name,
            "YAEP Thumbnail",
            false,
        );
        let (overlay_id, overlay_label) = if let Some(ref app) = app_for_overlay {
            let oid = thumbnail_webview_overlay::new_overlay_id();
            match thumbnail_webview_overlay::open_thumbnail_overlay_window(
                app,
                &oid,
                pid,
                thumbnail_window_hwnd,
            ) {
                Ok(label) => (oid, label),
                Err(e) => {
                    eprintln!("YAEP: thumbnail overlay webview: {e}");
                    (String::new(), String::new())
                }
            }
        } else {
            (String::new(), String::new())
        };
        diag::trace(
            "dwm",
            &format!("pid={pid} thumbnail_window_hwnd=0x{thumbnail_window_hwnd:x} overlay_label={overlay_label}"),
        );
        if thumbnail_window_hwnd != 0 {
            unsafe {
                SetWindowLongPtrW(
                    hwnd_from_isize(thumbnail_window_hwnd),
                    GWLP_USERDATA,
                    pid as isize,
                );
            }
        }
        RuntimeThumbnailWindow {
            thumbnail_window_hwnd,
            overlay_id,
            overlay_label,
            source_hwnd,
            thumbnail: 0,
            is_focused: false,
            window_title: title.clone(),
            config: config.clone(),
        }
    });

    if entry.thumbnail_window_hwnd == 0 {
        diag::trace("dwm", &format!("pid={pid} abort: thumbnail_window_hwnd==0"));
        return;
    }

    entry.window_title = title.clone();
    entry.config = config.clone();

    if entry.thumbnail != 0 && entry.source_hwnd != source_hwnd {
        unsafe {
            let _ = DwmUnregisterThumbnail(entry.thumbnail);
        }
        entry.thumbnail = 0;
    }
    entry.source_hwnd = source_hwnd;
    register_window_source(entry.thumbnail_window_hwnd, entry.source_hwnd);

    interaction_register(entry.thumbnail_window_hwnd, pid, title.clone(), 0);

    if entry.thumbnail == 0 {
        let dest = hwnd_from_isize(entry.thumbnail_window_hwnd);
        let source = hwnd_from_isize(entry.source_hwnd);
        let dest_ok = unsafe { IsWindow(Some(dest)) }.as_bool();
        let source_ok = unsafe { IsWindow(Some(source)) }.as_bool();
        if !dest_ok || !source_ok {
            diag::trace(
                "dwm",
                &format!(
                    "pid={pid} skip DwmRegisterThumbnail: IsWindow dest={dest_ok} source={source_ok}"
                ),
            );
            return;
        }
        diag::trace(
            "dwm",
            &format!("pid={pid} DwmRegisterThumbnail dest=0x{:x} source=0x{:x}", entry.thumbnail_window_hwnd, entry.source_hwnd),
        );
        match unsafe { DwmRegisterThumbnail(dest, source) } {
            Ok(thumb) => {
                diag::trace(
                    "dwm",
                    &format!("pid={pid} DwmRegisterThumbnail ok thumbnail={thumb:?}"),
                );
                entry.thumbnail = thumb;
            }
            Err(e) => {
                eprintln!("YAEP: DwmRegisterThumbnail failed pid={pid}: {e:?}");
                diag::trace("dwm", &format!("pid={pid} DwmRegisterThumbnail err: {e:?}"));
                return;
            }
        }
    }

    interaction_register(
        entry.thumbnail_window_hwnd,
        pid,
        title.clone(),
        entry.thumbnail,
    );

    set_window_title(entry.thumbnail_window_hwnd, &title);
    if let Some(app) = dwm_app_handle() {
        thumbnail_webview_overlay::set_overlay_window_title(&app, &entry.overlay_label, &title);
    }
    diag::trace(
        "dwm",
        &format!("pid={pid} before apply_thumbnail_container_geometry (DwmUpdateThumbnailProperties)"),
    );
    apply_thumbnail_container_geometry(
        entry.thumbnail_window_hwnd,
        entry.overlay_label.as_str(),
        &config,
        entry.thumbnail,
        thumbnail_window_is_hovered(entry.thumbnail_window_hwnd),
    );
    diag::trace("dwm", &format!("pid={pid} after apply_thumbnail_container_geometry"));
    unsafe {
        let _ = ShowWindow(hwnd_from_isize(entry.thumbnail_window_hwnd), SW_SHOWNA);
    }
    if let Some(app) = dwm_app_handle() {
        thumbnail_webview_overlay::show_overlay_window(&app, &entry.overlay_label);
    }
    // Keep ThumbnailOverlayWindow aligned above ThumbnailWindow (rect + Z); ShowWindow can reorder topmost.
    sync_overlay_bounds(entry.thumbnail_window_hwnd, entry.overlay_label.as_str());
    diag::trace("dwm", &format!("pid={pid} after ShowWindow; register path complete"));

    if let Some(app) = dwm_app_handle() {
        thumbnail_webview_overlay::emit_overlay_state(
            &app,
            &entry.overlay_label,
            &entry.overlay_id,
            pid,
            entry.is_focused,
            &entry.config,
            &entry.window_title,
        );
    }

    layout_snapshot_update(
        pid,
        entry.thumbnail_window_hwnd,
        entry.overlay_label.clone(),
        config.x,
        config.y,
        config.width,
        config.height,
    );
}

fn interaction_register(thumbnail_window_hwnd: isize, pid: u32, window_title: String, thumbnail: isize) {
    let mut m = interaction_map().lock().expect("interaction map lock poisoned");
    m.insert(
        thumbnail_window_hwnd,
        HostInteractionInfo {
            pid,
            window_title,
            thumbnail,
        },
    );
}

fn interaction_unregister(thumbnail_window_hwnd: isize) {
    let mut m = interaction_map().lock().expect("interaction map lock poisoned");
    m.remove(&thumbnail_window_hwnd);
}

fn unregister_runtime_thumbnail_locked(inner: &Arc<DwmInner>, pid: u32) {
    layout_snapshot_remove(pid);
    let mut state = inner.runtime.lock().expect("dwm runtime lock poisoned");
    let Some(runtime) = state.remove(&pid) else {
        return;
    };

    if runtime.thumbnail != 0 {
        unsafe {
            let _ = DwmUnregisterThumbnail(runtime.thumbnail);
        }
    }
    if runtime.thumbnail_window_hwnd != 0 {
        interaction_unregister(runtime.thumbnail_window_hwnd);
        unregister_window_source(runtime.thumbnail_window_hwnd);
    }
    if let Some(app) = dwm_app_handle() {
        thumbnail_webview_overlay::close_thumbnail_overlay_window(&app, &runtime.overlay_label);
    }
    if runtime.thumbnail_window_hwnd != 0 {
        unsafe {
            let _ = DestroyWindow(hwnd_from_isize(runtime.thumbnail_window_hwnd));
        }
    }
}

fn set_focused_thumbnail_locked(inner: &Arc<DwmInner>, focused_pid: Option<u32>) {
    let mut state = inner.runtime.lock().expect("dwm runtime lock poisoned");
    for (pid, runtime) in state.iter_mut() {
        let should_focus = Some(*pid) == focused_pid;
        if runtime.is_focused == should_focus {
            continue;
        }
        runtime.is_focused = should_focus;
        let focus_label = if should_focus { "[FOCUSED] " } else { "" };
        let label = format!("{focus_label}YAEP Overlay PID {pid}");
        if let Some(app) = dwm_app_handle() {
            thumbnail_webview_overlay::set_overlay_window_title(&app, &runtime.overlay_label, &label);
            sync_overlay_bounds(runtime.thumbnail_window_hwnd, runtime.overlay_label.as_str());
            thumbnail_webview_overlay::emit_overlay_state(
                &app,
                &runtime.overlay_label,
                &runtime.overlay_id,
                *pid,
                runtime.is_focused,
                &runtime.config,
                &runtime.window_title,
            );
        }
    }
}

fn register_thumbnail_window_class() -> &'static Vec<u16> {
    static CLASS_NAME: OnceLock<Vec<u16>> = OnceLock::new();
    static REGISTERED: OnceLock<u16> = OnceLock::new();

    let class_name = CLASS_NAME.get_or_init(|| to_wide("YAEP_ThumbnailWindow"));
    REGISTERED.get_or_init(|| {
        let hinst = unsafe { GetModuleHandleW(None).unwrap_or_default() };
        let h_cursor = unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() };
        let wc = WNDCLASSW {
            lpfnWndProc: Some(thumbnail_container_wnd_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: h_cursor,
            hInstance: hinst.into(),
            ..Default::default()
        };
        unsafe { RegisterClassW(&wc) }
    });
    class_name
}

/// Native **ThumbnailWindow** (Avalonia): frameless host; DWM draws the live preview into this HWND only.
fn create_thumbnail_window(class_name: &Vec<u16>, title: &str, click_through: bool) -> isize {
    let mut ex_style = WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0 | WS_EX_LAYERED.0;
    if click_through {
        ex_style |= WS_EX_TRANSPARENT.0;
    }
    let hinst = unsafe { GetModuleHandleW(None).unwrap_or_default() };
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(ex_style),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(to_wide(title).as_ptr()),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            Some(hinst.into()),
            None,
        )
    };
    match hwnd {
        Ok(handle) => handle.0 as isize,
        Err(_) => 0,
    }
}

fn thumbnail_drag_enabled() -> bool {
    let Ok(guard) = GLOBAL_DB.lock() else {
        return true;
    };
    let Some(db) = guard.as_ref() else {
        return true;
    };
    match db.get_app_setting("EnableThumbnailDragging".to_string()) {
        Ok(Some(v)) => v.eq_ignore_ascii_case("true"),
        Ok(None) => true,
        Err(_) => true,
    }
}

fn apply_thumbnail_container_geometry(
    thumbnail_window_hwnd: isize,
    overlay_label: &str,
    config: &ThumbnailConfig,
    thumbnail: isize,
    pointer_over_thumbnail: bool,
) {
    if thumbnail_window_hwnd == 0 {
        return;
    }
    let (w, h) = clamp_dims_i64(config.width, config.height);
    let x = clamp_pos_i64(config.x);
    let y = clamp_pos_i64(config.y);
    unsafe {
        let _ = SetWindowPos(
            hwnd_from_isize(thumbnail_window_hwnd),
            None,
            x,
            y,
            w,
            h,
            SWP_NOACTIVATE | SWP_NOZORDER,
        );
    }
    // DwmUpdateThumbnailProperties can disturb Z among WS_EX_TOPMOST peers; restack overlay *after*.
    update_thumbnail_properties_with_opacity(
        thumbnail_window_hwnd,
        thumbnail,
        config.opacity,
        pointer_over_thumbnail,
    );
    sync_overlay_bounds(thumbnail_window_hwnd, overlay_label);
}

/// DWM preview alpha: full while pointer is over the thumbnail window, else from settings.
fn opacity_byte(config_opacity: f64, pointer_over_thumbnail: bool) -> u8 {
    if pointer_over_thumbnail {
        return 255;
    }
    let o = config_opacity.clamp(0.0, 1.0);
    (o * 255.0).round().clamp(0.0, 255.0) as u8
}

fn update_thumbnail_properties_with_opacity(
    thumbnail_window_hwnd: isize,
    thumbnail: isize,
    config_opacity: f64,
    pointer_over_thumbnail: bool,
) {
    if thumbnail_window_hwnd == 0 || thumbnail == 0 {
        return;
    }
    let mut rect = RECT::default();
    let ok = unsafe { GetClientRect(hwnd_from_isize(thumbnail_window_hwnd), &mut rect) };
    if ok.is_err() {
        return;
    }
    let op = opacity_byte(config_opacity, pointer_over_thumbnail);
    let props = DwmThumbnailPropertiesAbi {
        dw_flags: DWM_THUMBNAIL_UPDATE_FLAGS,
        rc_destination: rect,
        rc_source: RECT::default(),
        opacity: op,
        _pad: [0; 3],
        f_visible: 1,
        f_source_client_area_only: 1,
    };
    let _ = unsafe {
        DwmUpdateThumbnailProperties(
            thumbnail,
            std::ptr::from_ref(&props).cast::<DWM_THUMBNAIL_PROPERTIES>(),
        )
    };
}

extern "system" fn thumbnail_container_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SETCURSOR {
        unsafe {
            if let Ok(c) = LoadCursorW(None, IDC_ARROW) {
                let _ = SetCursor(Some(c));
            }
        }
        return LRESULT(1);
    }

    if msg == WM_MOUSEMOVE {
        if handle_drag_move(hwnd) {
            return LRESULT(0);
        }
        unsafe {
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut tme);
        }
        let h = hwnd.0 as isize;
        if !thumbnail_window_is_hovered(h) {
            set_thumbnail_window_hovered(h, true);
            refresh_dwm_opacity_after_hover_change(h);
        }
    }

    if msg == WM_MOUSE_LEAVE {
        let h = hwnd.0 as isize;
        set_thumbnail_window_hovered(h, false);
        refresh_dwm_opacity_after_hover_change(h);
    }

    if msg == WM_MOUSEWHEEL {
        let ctrl = (wparam.0 as u32 & 0xffff) & MK_CONTROL_WPARAM != 0;
        if ctrl {
            if handle_thumbnail_wheel(hwnd, wparam) {
                return LRESULT(0);
            }
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    if msg == WM_RBUTTONDOWN {
        if handle_rbutton_down(hwnd) {
            return LRESULT(0);
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    if msg == WM_RBUTTONUP {
        if handle_rbutton_up(hwnd) {
            return LRESULT(0);
        }
    }

    if msg == WM_LBUTTONDOWN || msg == WM_LBUTTONDBLCLK {
        if let Some(source_hwnd) = resolve_source_for_container(hwnd.0 as isize) {
            let target = hwnd_from_isize(source_hwnd);
            unsafe {
                if !IsWindow(Some(target)).as_bool() {
                    return LRESULT(0);
                }
                if IsIconic(target).as_bool() {
                    let _ = ShowWindow(target, SW_RESTORE);
                }
                let _ = SetForegroundWindow(target);
            }
        }
        return LRESULT(0);
    }

    if msg == WM_MBUTTONDOWN {
        return LRESULT(0);
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// DWM opacity follows hover; restack overlay so chrome stays above the thumbnail after `DwmUpdateThumbnailProperties`.
fn refresh_dwm_opacity_after_hover_change(thumbnail_window_hwnd: isize) {
    let info = {
        let m = interaction_map().lock().ok();
        m.and_then(|g| g.get(&thumbnail_window_hwnd).cloned())
    };
    let Some(info) = info else {
        return;
    };
    let db_opt = GLOBAL_DB.lock().ok().and_then(|g| g.clone());
    let Some(db) = db_opt else {
        return;
    };
    let Some(profile_id) = db.active_profile_id() else {
        return;
    };
    let Ok(config) = db.resolve_thumbnail_config(profile_id, &info.window_title) else {
        return;
    };
    update_thumbnail_properties_with_opacity(
        thumbnail_window_hwnd,
        info.thumbnail,
        config.opacity,
        thumbnail_window_is_hovered(thumbnail_window_hwnd),
    );
    let overlay_label = layout_snapshot()
        .lock()
        .ok()
        .and_then(|m| m.get(&info.pid).map(|e| e.overlay_label.clone()))
        .unwrap_or_default();
    if !overlay_label.is_empty() {
        sync_overlay_bounds(thumbnail_window_hwnd, overlay_label.as_str());
    }
}

fn handle_rbutton_down(hwnd: HWND) -> bool {
    if !thumbnail_drag_enabled() {
        return false;
    }
    let thumbnail_window_hwnd = hwnd.0 as isize;
    let info = {
        let m = interaction_map().lock().ok();
        m.and_then(|g| g.get(&thumbnail_window_hwnd).cloned())
    };
    let Some(info) = info else {
        return false;
    };
    let group = unsafe { (GetKeyState(VK_CONTROL) as u16 & 0x8000) != 0 };
    let mut anchor = POINT::default();
    if unsafe { GetCursorPos(&mut anchor).is_err() } {
        return false;
    }
    let snap = layout_snapshot().lock().ok().map(|g| g.clone()).unwrap_or_default();
    let mut start_rects = HashMap::new();
    if group {
        for (pid, le) in &snap {
            start_rects.insert(*pid, (le.x, le.y, le.w, le.h));
        }
    } else if let Some(le) = snap.get(&info.pid) {
        start_rects.insert(info.pid, (le.x, le.y, le.w, le.h));
    } else {
        return false;
    }
    *drag_state().lock().expect("drag state lock poisoned") = Some(DragState {
        anchor,
        start_rects,
        primary_pid: info.pid,
        group,
    });
    unsafe {
        let _ = SetCapture(hwnd);
    }
    true
}

fn handle_drag_move(hwnd: HWND) -> bool {
    let cap = unsafe { GetCapture() };
    if cap.0 != hwnd.0 {
        return false;
    }
    let st = drag_state().lock().ok().and_then(|g| g.as_ref().cloned());
    let Some(st) = st else {
        return false;
    };
    let mut cur = POINT::default();
    if unsafe { GetCursorPos(&mut cur).is_err() } {
        return false;
    }
    let dx = cur.x - st.anchor.x;
    let dy = cur.y - st.anchor.y;
    let snap = layout_snapshot().lock().ok().map(|g| g.clone()).unwrap_or_default();
    for (pid, (sx, sy, sw, sh)) in &st.start_rects {
        let Some(le) = snap.get(pid) else {
            continue;
        };
        let nl = sx + dx;
        let nt = sy + dy;
        unsafe {
            let _ = SetWindowPos(
                hwnd_from_isize(le.thumbnail_window_hwnd),
                None,
                nl,
                nt,
                *sw,
                *sh,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }
        let thumb = interaction_map()
            .lock()
            .ok()
            .and_then(|m| m.get(&le.thumbnail_window_hwnd).map(|i| i.thumbnail));
        if let Some(t) = thumb {
            let db_opt = GLOBAL_DB.lock().ok().and_then(|g| g.clone());
            if let Some(db) = db_opt {
                if let Some(profile_id) = db.active_profile_id() {
                    let title = interaction_map()
                        .lock()
                        .ok()
                        .and_then(|m| m.get(&le.thumbnail_window_hwnd).map(|i| i.window_title.clone()));
                    if let Some(ref wt) = title {
                        if let Ok(cfg) = db.resolve_thumbnail_config(profile_id, wt) {
                            update_thumbnail_properties_with_opacity(
                                le.thumbnail_window_hwnd,
                                t,
                                cfg.opacity,
                                thumbnail_window_is_hovered(le.thumbnail_window_hwnd),
                            );
                        }
                    }
                }
            }
        }
        sync_overlay_bounds(le.thumbnail_window_hwnd, le.overlay_label.as_str());
        layout_snapshot_update(
            *pid,
            le.thumbnail_window_hwnd,
            le.overlay_label.clone(),
            nl as i64,
            nt as i64,
            *sw as i64,
            *sh as i64,
        );
    }
    true
}

fn handle_rbutton_up(hwnd: HWND) -> bool {
    let cap = unsafe { GetCapture() };
    if cap.0 != hwnd.0 {
        return false;
    }
    let ended = drag_state()
        .lock()
        .ok()
        .and_then(|mut g| g.take());
    unsafe {
        let _ = ReleaseCapture();
    }
    let Some(st) = ended else {
        return false;
    };
    persist_drag_results(&st);
    true
}

fn persist_drag_results(st: &DragState) {
    let Ok(guard) = GLOBAL_DB.lock() else {
        return;
    };
    let Some(db) = guard.as_ref() else {
        return;
    };
    let Some(profile_id) = db.active_profile_id() else {
        return;
    };
    let snap = layout_snapshot().lock().ok();
    let Some(ref layout) = snap else {
        return;
    };
    for pid in st.start_rects.keys() {
        if !st.group && *pid != st.primary_pid {
            continue;
        }
        let Some(le) = layout.get(pid) else {
            continue;
        };
        let title = interaction_map()
            .lock()
            .ok()
            .and_then(|m| m.get(&le.thumbnail_window_hwnd).map(|i| i.window_title.clone()));
        let Some(title) = title else {
            continue;
        };
        let Ok(mut cfg) = db.resolve_thumbnail_config(profile_id, &title) else {
            continue;
        };
        cfg.x = le.x as i64;
        cfg.y = le.y as i64;
        cfg.width = le.w as i64;
        cfg.height = le.h as i64;
        let _ = db.save_thumbnail_setting(profile_id, title, cfg);
    }
}

fn handle_thumbnail_wheel(hwnd: HWND, wparam: WPARAM) -> bool {
    let thumbnail_window_hwnd = hwnd.0 as isize;
    let info = match interaction_map()
        .lock()
        .ok()
        .and_then(|g| g.get(&thumbnail_window_hwnd).cloned())
    {
        Some(i) => i,
        None => return false,
    };
    let delta = ((wparam.0 >> 16) & 0xffff) as i16 as i32;
    let Ok(guard) = GLOBAL_DB.lock() else {
        return false;
    };
    let Some(db) = guard.as_ref() else {
        return false;
    };
    let Some(profile_id) = db.active_profile_id() else {
        return false;
    };
    let Ok(mut cfg) = db.resolve_thumbnail_config(profile_id, &info.window_title) else {
        return false;
    };
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd_from_isize(thumbnail_window_hwnd), &mut rect).is_err() } {
        return false;
    }
    let cur_w = (rect.right - rect.left).clamp(MIN_THUMB_W, MAX_THUMB_W);
    let cur_h = (rect.bottom - rect.top).clamp(MIN_THUMB_H, MAX_THUMB_H);
    let cx = (rect.left + rect.right) / 2;
    let cy = (rect.top + rect.bottom) / 2;
    let aspect = cur_h as f64 / cur_w.max(1) as f64;
    let step = if delta > 0 { 16 } else { -16 };
    let new_w = (cur_w + step).clamp(MIN_THUMB_W, MAX_THUMB_W);
    let mut new_h = (new_w as f64 * aspect).round() as i32;
    new_h = new_h.clamp(MIN_THUMB_H, MAX_THUMB_H);
    let new_x = cx - new_w / 2;
    let new_y = cy - new_h / 2;
    cfg.width = new_w as i64;
    cfg.height = new_h as i64;
    cfg.x = (new_x as i64).clamp(POS_LOW, POS_HIGH);
    cfg.y = (new_y as i64).clamp(POS_LOW, POS_HIGH);
    let overlay_label = layout_snapshot()
        .lock()
        .ok()
        .and_then(|m| m.get(&info.pid).map(|e| e.overlay_label.clone()))
        .unwrap_or_default();
    if let Err(e) = db.save_thumbnail_setting(profile_id, info.window_title.clone(), cfg.clone()) {
        eprintln!("YAEP: save after wheel resize: {e}");
        return false;
    }
    apply_thumbnail_container_geometry(
        thumbnail_window_hwnd,
        overlay_label.as_str(),
        &cfg,
        info.thumbnail,
        thumbnail_window_is_hovered(thumbnail_window_hwnd),
    );
    layout_snapshot_update(
        info.pid,
        thumbnail_window_hwnd,
        overlay_label,
        cfg.x,
        cfg.y,
        cfg.width,
        cfg.height,
    );
    true
}

fn sync_overlay_bounds(thumbnail_window_hwnd: isize, overlay_label: &str) {
    if thumbnail_window_hwnd == 0 || overlay_label.is_empty() {
        return;
    }
    let Some(app) = dwm_app_handle() else {
        return;
    };
    thumbnail_webview_overlay::sync_overlay_bounds_win(&app, thumbnail_window_hwnd, overlay_label);
}

fn set_window_title(hwnd: isize, title: &str) {
    if hwnd == 0 {
        return;
    }
    let wide = to_wide(title);
    unsafe {
        let _ = SetWindowTextW(hwnd_from_isize(hwnd), PCWSTR(wide.as_ptr()));
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn hwnd_from_isize(value: isize) -> HWND {
    HWND(value as *mut c_void)
}

fn source_map() -> &'static Mutex<HashMap<isize, isize>> {
    static SOURCE_MAP: OnceLock<Mutex<HashMap<isize, isize>>> = OnceLock::new();
    SOURCE_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_window_source(thumbnail_window_hwnd: isize, source_hwnd: isize) {
    let mut map = source_map().lock().expect("source map lock poisoned");
    map.insert(thumbnail_window_hwnd, source_hwnd);
}

fn unregister_window_source(thumbnail_window_hwnd: isize) {
    let mut map = source_map().lock().expect("source map lock poisoned");
    map.remove(&thumbnail_window_hwnd);
}

fn resolve_source_for_container(thumbnail_window_hwnd: isize) -> Option<isize> {
    let map = source_map().lock().expect("source map lock poisoned");
    map.get(&thumbnail_window_hwnd).copied()
}