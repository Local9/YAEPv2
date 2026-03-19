use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};

use tauri::AppHandle;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
    DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_VISIBLE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetWindowRect, IsIconic,
    IsWindow, LoadCursorW, RegisterClassW, SetCursor, SetForegroundWindow, SetWindowPos,
    SetWindowTextW, ShowWindow, CW_USEDEFAULT, IDC_ARROW, SW_RESTORE, SW_SHOWNA, SWP_NOACTIVATE,
    SWP_NOZORDER, WINDOW_EX_STYLE, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MBUTTONDOWN,
    WM_RBUTTONDOWN, WM_SETCURSOR, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::PCWSTR;

#[derive(Debug, Clone)]
struct RuntimeThumbnailWindow {
    host_hwnd: isize,
    overlay_hwnd: isize,
    source_hwnd: isize,
    thumbnail: isize,
    is_focused: bool,
}

struct DwmInner {
    runtime: Mutex<HashMap<u32, RuntimeThumbnailWindow>>,
    app_handle: Mutex<Option<AppHandle>>,
}

/// DWM thumbnail hosts must be created and messaged on the thread that runs a Win32 message loop.
/// Tauri drives that on the main thread; thumbnail refresh runs on Tokio workers, so we marshal
/// all window/DWM work through [`AppHandle::run_on_main_thread`].
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
        }
    }
}

impl DwmService {
    pub fn sync_thumbnail_graph(&self) {}

    pub fn set_app_handle(&self, app: AppHandle) {
        *self.inner.app_handle.lock().expect("dwm app_handle lock poisoned") = Some(app);
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
            return;
        }
        let title = title.to_string();
        let inner = self.inner.clone();
        self.run_on_main(move || {
            register_runtime_thumbnail_locked(&inner, pid, source_hwnd, title);
        });
    }

    pub fn unregister_runtime_thumbnail(&self, pid: u32) {
        let inner = self.inner.clone();
        self.run_on_main(move || {
            unregister_runtime_thumbnail_locked(&inner, pid);
        });
    }

    pub fn set_focused_thumbnail(&self, focused_pid: Option<u32>) {
        let inner = self.inner.clone();
        self.run_on_main(move || {
            set_focused_thumbnail_locked(&inner, focused_pid);
        });
    }
}

fn register_runtime_thumbnail_locked(
    inner: &Arc<DwmInner>,
    pid: u32,
    source_hwnd: isize,
    title: String,
) {
    let class_name = register_thumbnail_window_class();
    let mut state = inner.runtime.lock().expect("dwm runtime lock poisoned");

    let entry = state.entry(pid).or_insert_with(|| {
        let hwnd = create_plain_window(class_name, "YAEP Thumbnail", false);
        let overlay = create_plain_window(class_name, "YAEP Overlay", true);
        RuntimeThumbnailWindow {
            host_hwnd: hwnd,
            overlay_hwnd: overlay,
            source_hwnd,
            thumbnail: 0,
            is_focused: false,
        }
    });

    if entry.host_hwnd == 0 {
        return;
    }

    if entry.thumbnail != 0 && entry.source_hwnd != source_hwnd {
        unsafe {
            let _ = DwmUnregisterThumbnail(entry.thumbnail);
        }
        entry.thumbnail = 0;
    }
    entry.source_hwnd = source_hwnd;
    register_window_source(entry.host_hwnd, entry.source_hwnd);

    if entry.thumbnail == 0 {
        let register_result = unsafe {
            DwmRegisterThumbnail(
                hwnd_from_isize(entry.host_hwnd),
                hwnd_from_isize(entry.source_hwnd),
            )
        };
        if let Ok(thumb) = register_result {
            entry.thumbnail = thumb;
        } else {
            return;
        }
    }

    set_window_title(entry.host_hwnd, &title);
    set_window_title(entry.overlay_hwnd, &title);
    update_thumbnail_properties(entry.host_hwnd, entry.thumbnail);
    sync_overlay_bounds(entry.host_hwnd, entry.overlay_hwnd);
    unsafe {
        let _ = ShowWindow(hwnd_from_isize(entry.host_hwnd), SW_SHOWNA);
        let _ = ShowWindow(hwnd_from_isize(entry.overlay_hwnd), SW_SHOWNA);
    }
}

fn unregister_runtime_thumbnail_locked(inner: &Arc<DwmInner>, pid: u32) {
    let mut state = inner.runtime.lock().expect("dwm runtime lock poisoned");
    let Some(runtime) = state.remove(&pid) else {
        return;
    };

    if runtime.thumbnail != 0 {
        unsafe {
            let _ = DwmUnregisterThumbnail(runtime.thumbnail);
        }
    }
    if runtime.host_hwnd != 0 {
        unregister_window_source(runtime.host_hwnd);
        unsafe {
            let _ = DestroyWindow(hwnd_from_isize(runtime.host_hwnd));
        }
    }
    if runtime.overlay_hwnd != 0 {
        unsafe {
            let _ = DestroyWindow(hwnd_from_isize(runtime.overlay_hwnd));
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
        set_window_title(runtime.overlay_hwnd, &label);
        sync_overlay_bounds(runtime.host_hwnd, runtime.overlay_hwnd);
    }
}

fn register_thumbnail_window_class() -> &'static Vec<u16> {
    static CLASS_NAME: OnceLock<Vec<u16>> = OnceLock::new();
    static REGISTERED: OnceLock<u16> = OnceLock::new();

    let class_name = CLASS_NAME.get_or_init(|| to_wide("YAEP_ThumbnailHostWindow"));
    REGISTERED.get_or_init(|| {
        let h_cursor = unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() };
        let wc = WNDCLASSW {
            lpfnWndProc: Some(host_window_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: h_cursor,
            ..Default::default()
        };
        unsafe { RegisterClassW(&wc) }
    });
    class_name
}

fn create_plain_window(class_name: &Vec<u16>, title: &str, click_through: bool) -> isize {
    let mut ex_style = WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0 | WS_EX_LAYERED.0;
    if click_through {
        ex_style |= WS_EX_TRANSPARENT.0;
    }
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
            None,
            None,
        )
    };
    match hwnd {
        Ok(handle) => handle.0 as isize,
        Err(_) => 0,
    }
}

extern "system" fn host_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SETCURSOR {
        unsafe {
            if let Ok(c) = LoadCursorW(None, IDC_ARROW) {
                let _ = SetCursor(c);
            }
        }
        return LRESULT(1);
    }

    if msg == WM_LBUTTONDOWN
        || msg == WM_LBUTTONDBLCLK
        || msg == WM_RBUTTONDOWN
        || msg == WM_MBUTTONDOWN
    {
        if let Some(source_hwnd) = resolve_source_for_host(hwnd.0 as isize) {
            let target = hwnd_from_isize(source_hwnd);
            unsafe {
                if !IsWindow(target).as_bool() {
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
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn update_thumbnail_properties(host_hwnd: isize, thumbnail: isize) {
    if host_hwnd == 0 || thumbnail == 0 {
        return;
    }
    let mut rect = RECT::default();
    let ok = unsafe { GetClientRect(hwnd_from_isize(host_hwnd), &mut rect) };
    if ok.is_err() {
        return;
    }

    let props = DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_VISIBLE | DWM_TNP_RECTDESTINATION | DWM_TNP_OPACITY,
        fVisible: BOOL(1),
        rcDestination: rect,
        opacity: 255,
        ..Default::default()
    };
    let _ = unsafe { DwmUpdateThumbnailProperties(thumbnail, &props) };
}

fn sync_overlay_bounds(host_hwnd: isize, overlay_hwnd: isize) {
    if host_hwnd == 0 || overlay_hwnd == 0 {
        return;
    }
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd_from_isize(host_hwnd), &mut rect) }.is_err() {
        return;
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    unsafe {
        let _ = SetWindowPos(
            hwnd_from_isize(overlay_hwnd),
            None,
            rect.left,
            rect.top,
            width,
            height,
            SWP_NOACTIVATE | SWP_NOZORDER,
        );
    }
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

fn register_window_source(host_hwnd: isize, source_hwnd: isize) {
    let mut map = source_map().lock().expect("source map lock poisoned");
    map.insert(host_hwnd, source_hwnd);
}

fn unregister_window_source(host_hwnd: isize) {
    let mut map = source_map().lock().expect("source map lock poisoned");
    map.remove(&host_hwnd);
}

fn resolve_source_for_host(host_hwnd: isize) -> Option<isize> {
    let map = source_map().lock().expect("source map lock poisoned");
    map.get(&host_hwnd).copied()
}
