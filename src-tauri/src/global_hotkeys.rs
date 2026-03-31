//! Win32 `RegisterHotKey` on a dedicated thread + `WM_HOTKEY` dispatch.
//! `WH_KEYBOARD_LL` must not swallow keys for registered combos: that prevents `WM_HOTKEY` when
//! another process owns the foreground (non-injected input). Hotkeys are not delivered to the
//! foreground app by `RegisterHotKey` alone (MSDN).

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleExW, GetModuleHandleW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
};
use windows::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RID_INPUT, RIM_TYPEKEYBOARD, RIDEV_INPUTSINK, RIDEV_REMOVE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL,
    MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, VK_0, VK_A, VK_BACK, VK_DELETE, VK_DOWN, VK_END,
    VK_ESCAPE, VK_F1, VK_HOME, VK_INSERT, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_LEFT,
    VK_NEXT, VK_NUMPAD0, VK_PRIOR, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_RETURN, VK_RIGHT,
    VK_SHIFT, VK_CONTROL, VK_MENU, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT,
    LLKHF_INJECTED, LLKHF_UP, PeekMessageW, PM_REMOVE,
    PostThreadMessageW, RegisterClassW, SetWindowsHookExW, TranslateMessage, HHOOK, HWND_MESSAGE, MSG,
    RI_KEY_BREAK, WH_KEYBOARD_LL, WM_INPUT, WM_KEYDOWN, WM_KEYUP, WM_NULL, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WINDOW_EX_STYLE, WINDOW_STYLE, WM_HOTKEY, WNDCLASSW,
};

use windows::core::PCWSTR;

use crate::db::DbService;
use crate::diag;
use crate::AppState;

static REFRESH_TX: Mutex<Option<mpsc::Sender<()>>> = Mutex::new(None);
static THREAD_STARTED: AtomicBool = AtomicBool::new(false);
/// Thread that owns the message-only window + hook; used to wake `GetMessageW` on refresh.
static HOTKEY_MSG_THREAD_ID: AtomicU32 = AtomicU32::new(0);
/// Message-only window receiving `WM_HOTKEY` / `WM_INPUT` (`HWND` bits).
static HOTKEY_MESSAGE_HWND: AtomicUsize = AtomicUsize::new(0);

/// Low-level keyboard hook handle (`HHOOK` bits; 0 = not installed).
static LL_KEYBOARD_HOOK_HANDLE: AtomicUsize = AtomicUsize::new(0);
/// Every entry into `low_level_keyboard_proc` (proves Windows invoked our hook).
static LL_HOOK_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);
/// Increments when `n_code >= 0` (normal HC_ACTION path).
static LL_HOOK_NONNEG_CALLS: AtomicU64 = AtomicU64::new(0);
static LL_HOOK_FIRST_NCODE_LOGGED: AtomicBool = AtomicBool::new(false);

struct CaptureState {
    active: bool,
    capture_type: String,
    target_id: Option<i64>,
    /// Non-modifier vk awaiting key-up to finalize capture.
    pending_capture_vk: Option<u32>,
}

static CAPTURE: Mutex<CaptureState> = Mutex::new(CaptureState {
    active: false,
    capture_type: String::new(),
    target_id: None,
    pending_capture_vk: None,
});

pub fn begin_hotkey_capture(capture_type: String, target_id: Option<i64>) {
    let calls_before = LL_HOOK_ENTRY_COUNT.load(Ordering::Relaxed);
    let hook_bits = LL_KEYBOARD_HOOK_HANDLE.load(Ordering::SeqCst);
    diag::trace(
        "hotkeys",
        &format!(
            "capture_begin type={capture_type} target_id={target_id:?} ll_hook_bits=0x{hook_bits:x} hook_entry_before={calls_before}",
        ),
    );
    if hook_bits == 0 {
        diag::trace(
            "hotkeys",
            "capture_begin: WH_KEYBOARD_LL not installed; using raw-input sink for capture if hwnd is ready",
        );
    }
    if let Ok(mut g) = CAPTURE.lock() {
        g.active = true;
        g.capture_type = capture_type;
        g.target_id = target_id;
        g.pending_capture_vk = None;
        diag::trace("hotkeys", "capture_begin: CAPTURE.active=true");
        drop(g);
        let hwnd_bits = HOTKEY_MESSAGE_HWND.load(Ordering::Acquire);
        if hwnd_bits != 0 {
            let hwnd = HWND(hwnd_bits as *mut std::ffi::c_void);
            let ok = unsafe { register_capture_raw_keyboard_sink(hwnd) };
            diag::trace(
                "hotkeys",
                &format!(
                    "capture_begin: RegisterRawInputDevices(KEYBOARD,INPUTSINK) hwnd=0x{hwnd_bits:x} ok={ok}",
                ),
            );
            if ok {
                let tid = HOTKEY_MSG_THREAD_ID.load(Ordering::Acquire);
                if tid != 0 {
                    unsafe {
                        let _ = PostThreadMessageW(tid, WM_NULL, WPARAM(0), LPARAM(0));
                    }
                }
            }
        } else {
            diag::trace(
                "hotkeys",
                "capture_begin: HOTKEY_MESSAGE_HWND unset; raw keyboard capture unavailable",
            );
        }
    } else {
        diag::trace(
            "hotkeys",
            "capture_begin: CAPTURE mutex poisoned; state not updated",
        );
    }
}

pub fn end_hotkey_capture() {
    let unreg_ok = unsafe { unregister_capture_raw_keyboard() };
    diag::trace(
        "hotkeys",
        &format!("capture_end RegisterRawInputDevices(REMOVE keyboard) ok={unreg_ok}"),
    );
    let tid = HOTKEY_MSG_THREAD_ID.load(Ordering::Acquire);
    if tid != 0 {
        unsafe {
            let _ = PostThreadMessageW(tid, WM_NULL, WPARAM(0), LPARAM(0));
        }
    }
    let entries = LL_HOOK_ENTRY_COUNT.load(Ordering::Relaxed);
    let nonneg = LL_HOOK_NONNEG_CALLS.load(Ordering::Relaxed);
    diag::trace(
        "hotkeys",
        &format!(
            "capture_end hook_entry_total={entries} hook_nonneg_total={nonneg} (LL hook; raw path ignores this)",
        ),
    );
    if let Ok(mut g) = CAPTURE.lock() {
        g.active = false;
        g.capture_type.clear();
        g.target_id = None;
        g.pending_capture_vk = None;
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileChangedPayload {
    profile_id: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyCapturedPayload {
    value: String,
    capture_type: String,
    target_id: Option<i64>,
}

#[derive(Clone)]
enum HotkeyAction {
    ActivateProfile(i64),
    CycleGroup { group_id: i64, forward: bool },
    OpenMumbleLink(i64),
}

pub fn request_refresh() {
    if let Ok(g) = REFRESH_TX.lock() {
        if let Some(tx) = g.as_ref() {
            let _ = tx.send(());
        }
    }
    let tid = HOTKEY_MSG_THREAD_ID.load(Ordering::Acquire);
    if tid != 0 {
        unsafe {
            let _ = PostThreadMessageW(tid, WM_NULL, WPARAM(0), LPARAM(0));
        }
    }
}

pub fn spawn_thread(db: Arc<DbService>) {
    if THREAD_STARTED.swap(true, Ordering::SeqCst) {
        diag::trace("hotkeys", "spawn_thread: already started");
        return;
    }
    diag::trace("hotkeys", "spawn_thread: starting message loop thread");
    let (tx, rx) = mpsc::channel();
    *REFRESH_TX.lock().expect("refresh tx lock") = Some(tx);
    std::thread::spawn(move || run_message_loop(db, rx));
}

fn hotkey_class_name() -> &'static Vec<u16> {
    static NAME: OnceLock<Vec<u16>> = OnceLock::new();
    NAME.get_or_init(|| to_wide("YAEP_GlobalHotkeyHost"))
}

fn run_message_loop(db: Arc<DbService>, refresh_rx: mpsc::Receiver<()>) {
    diag::trace("hotkeys", "run_message_loop: thread entered");
    static CLASS_INIT: OnceLock<()> = OnceLock::new();
    let class = hotkey_class_name();
    CLASS_INIT.get_or_init(|| {
        diag::trace("hotkeys", "run_message_loop: RegisterClassW (once)");
        let hinst = unsafe { GetModuleHandleW(None).unwrap_or_default() };
        let wc = WNDCLASSW {
            lpfnWndProc: Some(hotkey_wnd_proc),
            lpszClassName: PCWSTR(class.as_ptr()),
            hInstance: hinst.into(),
            ..Default::default()
        };
        unsafe {
            let _ = RegisterClassW(&wc);
        }
    });
    let hinst = unsafe { GetModuleHandleW(None).unwrap_or_default() };
    let hwnd = match unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class.as_ptr()),
            PCWSTR(to_wide("").as_ptr()),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(hinst.into()),
            None,
        )
    } {
        Ok(h) => h,
        Err(e) => {
            diag::trace("hotkeys", &format!("CreateWindowExW(HWND_MESSAGE) failed: {e:?}"));
            return;
        }
    };
    HOTKEY_MSG_THREAD_ID.store(unsafe { GetCurrentThreadId() }, Ordering::Release);
    HOTKEY_MESSAGE_HWND.store(hwnd.0 as usize, Ordering::Release);
    diag::trace(
        "hotkeys",
        &format!("message window ok hwnd=0x{:x}", hwnd.0 as isize),
    );

    unsafe {
        let try_install = |hmod: Option<HINSTANCE>| {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), hmod, 0)
        };

        let mut from_addr_module = HMODULE::default();
        let from_addr_ok = GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            PCWSTR(low_level_keyboard_proc as usize as *mut u16),
            &mut from_addr_module,
        )
        .is_ok()
            && !from_addr_module.is_invalid();

        if !from_addr_ok {
            diag::trace(
                "hotkeys",
                "GetModuleHandleExW(FROM_ADDRESS, low_level_keyboard_proc) failed; will try hMod NULL then main exe",
            );
        }

        let mut installed = false;
        if from_addr_ok {
            match try_install(Some(HINSTANCE(from_addr_module.0))) {
                Ok(h) => {
                    LL_KEYBOARD_HOOK_HANDLE.store(h.0 as usize, Ordering::SeqCst);
                    diag::trace(
                        "hotkeys",
                        &format!(
                            "WH_KEYBOARD_LL hook installed (hMod=FROM_ADDRESS) hhk=0x{:x}",
                            h.0 as usize
                        ),
                    );
                    installed = true;
                }
                Err(e) => {
                    diag::trace(
                        "hotkeys",
                        &format!(
                            "SetWindowsHookExW(WH_KEYBOARD_LL) hMod=FROM_ADDRESS failed: {e:?}; try hMod NULL"
                        ),
                    );
                }
            }
        }

        if !installed {
            match try_install(None) {
                Ok(h) => {
                    LL_KEYBOARD_HOOK_HANDLE.store(h.0 as usize, Ordering::SeqCst);
                    diag::trace(
                        "hotkeys",
                        &format!("WH_KEYBOARD_LL hook installed (hMod=NULL) hhk=0x{:x}", h.0 as usize),
                    );
                    installed = true;
                }
                Err(e1) => {
                    diag::trace(
                        "hotkeys",
                        &format!(
                            "SetWindowsHookExW(WH_KEYBOARD_LL) hMod=NULL failed: {e1:?}; retry GetModuleHandle(main)"
                        ),
                    );
                    let hmod = GetModuleHandleW(None).unwrap_or_default();
                    match try_install(Some(HINSTANCE(hmod.0))) {
                        Ok(h) => {
                            LL_KEYBOARD_HOOK_HANDLE.store(h.0 as usize, Ordering::SeqCst);
                            diag::trace(
                                "hotkeys",
                                &format!(
                                    "WH_KEYBOARD_LL hook installed (hMod=GetModuleHandle) hhk=0x{:x}",
                                    h.0 as usize
                                ),
                            );
                            installed = true;
                        }
                        Err(e2) => {
                            diag::trace(
                                "hotkeys",
                                &format!("SetWindowsHookExW(WH_KEYBOARD_LL) retry failed: {e2:?}"),
                            );
                        }
                    }
                }
            }
        }

        if !installed {
            diag::trace(
                "hotkeys",
                "WH_KEYBOARD_LL not installed: hotkey capture and combo suppress will not run (RegisterHotKey still works)",
            );
        }
    }

    let mut registered: Vec<i32> = Vec::new();
    refresh_registrations(hwnd, db.as_ref(), &mut registered);
    diag::trace("hotkeys", "entering GetMessageW loop");

    loop {
        while let Ok(()) = refresh_rx.try_recv() {
            refresh_registrations(hwnd, db.as_ref(), &mut registered);
        }
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        let mut msg = MSG::default();
        let gm = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if gm.0 == 0 {
            break;
        }
        if gm.0 == -1 {
            continue;
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn register_capture_raw_keyboard_sink(target: HWND) -> bool {
    let rid = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x06,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: target,
    };
    RegisterRawInputDevices(&[rid], std::mem::size_of::<RAWINPUTDEVICE>() as u32).is_ok()
}

unsafe fn unregister_capture_raw_keyboard() -> bool {
    let rid = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x06,
        dwFlags: RIDEV_REMOVE,
        hwndTarget: HWND::default(),
    };
    RegisterRawInputDevices(&[rid], std::mem::size_of::<RAWINPUTDEVICE>() as u32).is_ok()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HotkeyCaptureDisposition {
    Swallow,
    PassThrough,
}

/// When global capture is active, handle key-up / key-down. `None` if capture is off.
fn try_dispatch_hotkey_capture(
    vk: u32,
    key_up: bool,
    key_down: bool,
    diag_scan: Option<u32>,
    diag_source: &'static str,
) -> Option<HotkeyCaptureDisposition> {
    let scan_txt = diag_scan
        .map(|s| format!("scan=0x{s:x} "))
        .unwrap_or_default();
    if let Ok(mut g) = CAPTURE.lock() {
        if !g.active {
            return None;
        }
        let is_mod = is_modifier_vk(vk);
        diag::trace(
            "hotkeys",
            &format!(
                "{diag_source} vk=0x{vk:x} {scan_txt}up={key_up} down={key_down} modifier={is_mod} pending={:?}",
                g.pending_capture_vk
            ),
        );
        if is_mod {
            return Some(HotkeyCaptureDisposition::PassThrough);
        }
        if key_up {
            if g.pending_capture_vk == Some(vk) {
                let built = build_normalized_hotkey(vk);
                let Some(value) = built else {
                    diag::trace(
                        "hotkeys",
                        &format!(
                            "{diag_source} capture_key_up vk=0x{vk:x} normalize_failed; capture still active (try another key)",
                        ),
                    );
                    g.pending_capture_vk = None;
                    return Some(HotkeyCaptureDisposition::Swallow);
                };
                let payload = HotkeyCapturedPayload {
                    value,
                    capture_type: g.capture_type.clone(),
                    target_id: g.target_id,
                };
                g.active = false;
                g.pending_capture_vk = None;
                g.capture_type.clear();
                g.target_id = None;
                let app_for_emit = HOTKEY_APP_HANDLE.get().cloned();
                drop(g);
                if app_for_emit.is_none() {
                    diag::trace("hotkeys", "capture_emit skipped: HOTKEY_APP_HANDLE unset");
                }
                if let Some(app_ref) = app_for_emit {
                    diag::trace(
                        "hotkeys",
                        &format!(
                            "capture_emit type={} target_id={:?} value={}",
                            payload.capture_type, payload.target_id, payload.value
                        ),
                    );
                    let app_emit = app_ref.clone();
                    let _ = app_ref.run_on_main_thread(move || {
                        let _ = app_emit.emit("hotkeyCaptured", payload);
                    });
                }
                return Some(HotkeyCaptureDisposition::Swallow);
            }
            diag::trace(
                "hotkeys",
                &format!(
                    "{diag_source} capture_key_up vk=0x{vk:x} pending_mismatch pending={:?}",
                    g.pending_capture_vk
                ),
            );
            return Some(HotkeyCaptureDisposition::PassThrough);
        }
        if key_down {
            if g.pending_capture_vk.is_none() {
                g.pending_capture_vk = Some(vk);
                diag::trace(
                    "hotkeys",
                    &format!("{diag_source} capture_key_down vk=0x{vk:x} pending_set"),
                );
            } else {
                diag::trace(
                    "hotkeys",
                    &format!(
                        "{diag_source} capture_key_down vk=0x{vk:x} ignored (pending={:?})",
                        g.pending_capture_vk
                    ),
                );
            }
            return Some(HotkeyCaptureDisposition::Swallow);
        }
        diag::trace(
            "hotkeys",
            &format!("{diag_source} vk=0x{vk:x} pass (not up/down branch)"),
        );
        return Some(HotkeyCaptureDisposition::PassThrough);
    } else if diag::enabled() {
        diag::trace(
            "hotkeys",
            "capture: CAPTURE.lock() failed (mutex poisoned); hotkey capture may be stuck",
        );
    }
    None
}

fn try_handle_wm_input_capture(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    let hraw = HRAWINPUT(lparam.0 as *mut std::ffi::c_void);
    if hraw.is_invalid() {
        return None;
    }
    let header_sz = std::mem::size_of::<RAWINPUTHEADER>() as u32;
    let mut size: u32 = 0;
    let q = unsafe { GetRawInputData(hraw, RID_INPUT, None, &mut size, header_sz) };
    if q == u32::MAX || size == 0 || size > 16 * 1024 {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    let mut got_size = size;
    let n = unsafe {
        GetRawInputData(
            hraw,
            RID_INPUT,
            Some(buf.as_mut_ptr().cast()),
            &mut got_size,
            header_sz,
        )
    };
    if n == u32::MAX || n == 0 {
        return None;
    }
    let raw = unsafe { &*(buf.as_ptr().cast::<RAWINPUT>()) };
    if raw.header.dwType != RIM_TYPEKEYBOARD.0 {
        return None;
    }
    let kb = unsafe { raw.data.keyboard };
    let vk = kb.VKey as u32;
    if vk == 0xE7 {
        return None;
    }
    let key_up = (kb.Flags as u32 & RI_KEY_BREAK) != 0;
    let key_down = kb.Message == WM_KEYDOWN || kb.Message == WM_SYSKEYDOWN;
    let disp = try_dispatch_hotkey_capture(
        vk,
        key_up,
        key_down,
        Some(kb.MakeCode as u32),
        "capture_raw",
    )?;
    Some(match disp {
        HotkeyCaptureDisposition::Swallow => LRESULT(0),
        HotkeyCaptureDisposition::PassThrough => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    })
}

extern "system" fn hotkey_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_INPUT {
        if let Some(lr) = try_handle_wm_input_capture(hwnd, msg, wparam, lparam) {
            return lr;
        }
    }
    if msg == WM_HOTKEY {
        let id = wparam.0 as i32;
        if let Some(action) = take_action_for_id(id) {
            dispatch_hotkey_action(action);
        }
        return LRESULT(0);
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

static ID_TABLE: OnceLock<Mutex<Vec<(i32, HotkeyAction)>>> = OnceLock::new();

fn id_table() -> &'static Mutex<Vec<(i32, HotkeyAction)>> {
    ID_TABLE.get_or_init(|| Mutex::new(Vec::new()))
}

fn take_action_for_id(id: i32) -> Option<HotkeyAction> {
    let t = id_table().lock().ok()?;
    t.iter()
        .find(|(i, _)| *i == id)
        .map(|(_, a)| a.clone())
}

fn dispatch_hotkey_action(action: HotkeyAction) {
    let Some(app) = HOTKEY_APP_HANDLE.get() else {
        return;
    };
    let app = app.clone();
    let app_on_main = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(state) = app_on_main.try_state::<AppState>() else {
            return;
        };
        match action {
            HotkeyAction::ActivateProfile(profile_id) => {
                if state.db.set_active_profile(profile_id).is_err() {
                    return;
                }
                state.thumbnail_service.stop();
                state.thumbnail_service.start(
                    app_on_main.clone(),
                    state.db.clone(),
                    state.window_service.clone(),
                    state.dwm.clone(),
                );
                let _ = app_on_main.emit(
                    "profileChanged",
                    ProfileChangedPayload { profile_id },
                );
                request_refresh();
            }
            HotkeyAction::CycleGroup { group_id, forward } => {
                let direction = if forward { "forward" } else { "backward" };
                if let Err(e) = crate::cycle_client_group_internal(
                    &*state,
                    group_id,
                    direction,
                    true,
                ) {
                    diag::trace("hotkeys", &format!("cycle_client_group WM_HOTKEY failed: {e}"));
                }
            }
            HotkeyAction::OpenMumbleLink(link_id) => {
                let _ = crate::open_mumble_link_internal(&*state, link_id);
            }
        }
    });
}

static HOTKEY_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(app: &AppHandle) {
    let _ = HOTKEY_APP_HANDLE.set(app.clone());
}

fn refresh_registrations(hwnd: HWND, db: &DbService, registered: &mut Vec<i32>) {
    for id in registered.drain(..) {
        unsafe {
            let _ = UnregisterHotKey(Some(hwnd), id);
        }
    }
    let mut table = id_table().lock().expect("id table lock poisoned");
    table.clear();

    let mut next_id = 1i32;
    let mut push = |table: &mut Vec<(i32, HotkeyAction)>, id: &mut i32, hk: &str, action: HotkeyAction| {
        if hk.trim().is_empty() {
            return;
        }
        let Some((mods, vk)) = parse_hotkey(hk) else {
            diag::trace(
                "hotkeys",
                &format!("RegisterHotKey skipped (parse/validate failed): {hk:?}"),
            );
            return;
        };
        unsafe {
            if let Err(e) = RegisterHotKey(Some(hwnd), *id, mods, vk) {
                diag::trace(
                    "hotkeys",
                    &format!(
                        "RegisterHotKey failed id={} hk={hk:?} mods=0x{:x} vk=0x{vk:x} err={e:?}",
                        *id,
                        mods.0
                    ),
                );
                return;
            }
        }
        table.push((*id, action));
        registered.push(*id);
        *id += 1;
    };

    if let Ok(profiles) = db.get_profiles() {
        for p in profiles {
            if p.deleted_at.is_some() {
                continue;
            }
            push(
                &mut table,
                &mut next_id,
                &p.switch_hotkey,
                HotkeyAction::ActivateProfile(p.id),
            );
        }
    }

    if let Some(pid) = db.active_profile_id() {
        if let Ok(groups) = db.get_client_groups(pid) {
            for g in groups {
                push(
                    &mut table,
                    &mut next_id,
                    &g.cycle_forward_hotkey,
                    HotkeyAction::CycleGroup {
                        group_id: g.id,
                        forward: true,
                    },
                );
                push(
                    &mut table,
                    &mut next_id,
                    &g.cycle_backward_hotkey,
                    HotkeyAction::CycleGroup {
                        group_id: g.id,
                        forward: false,
                    },
                );
            }
        }
    }

    if let Ok(links) = db.get_mumble_links() {
        for link in links {
            push(
                &mut table,
                &mut next_id,
                &link.hotkey,
                HotkeyAction::OpenMumbleLink(link.id),
            );
        }
    }
}

fn parse_hotkey(raw: &str) -> Option<(HOT_KEY_MODIFIERS, u32)> {
    let svc = crate::hotkeys::HotkeyService::default();
    let normalized = svc.validate_hotkey(raw).ok()?;
    if normalized.is_empty() {
        return None;
    }
    let parts: Vec<&str> = normalized.split('+').collect();
    let mut mods = MOD_NOREPEAT;
    let mut key: Option<&str> = None;
    for p in parts {
        match p {
            "Ctrl" => mods |= MOD_CONTROL,
            "Alt" => mods |= MOD_ALT,
            "Shift" => mods |= MOD_SHIFT,
            "Win" => mods |= MOD_WIN,
            other => key = Some(other),
        }
    }
    let key = key?;
    let vk = parse_vk_token(key)?;
    Some((mods, vk))
}

fn parse_vk_token(key: &str) -> Option<u32> {
    let u = key.to_uppercase();
    if let Some(n) = u.strip_prefix('F') {
        if let Ok(v) = n.parse::<u8>() {
            if (1..=24).contains(&v) {
                return Some((VK_F1.0 + (v as u16) - 1) as u32);
            }
        }
    }
    if let Some(n) = u.strip_prefix("VK") {
        if let Ok(v) = n.parse::<u32>() {
            if (1..=255).contains(&v) {
                return Some(v);
            }
        }
    }
    if let Some(n) = u.strip_prefix("NUMPAD") {
        if let Ok(d) = n.parse::<u8>() {
            if d <= 9 {
                return Some((VK_NUMPAD0.0 + d as u16) as u32);
            }
        }
    }
    if u.len() == 1 {
        let c = u.chars().next()?;
        if c.is_ascii_digit() {
            let d = c.to_digit(10)? as u16;
            return Some((VK_0.0 + d) as u32);
        }
        if c.is_ascii_uppercase() {
            return Some((VK_A.0 + (c as u16 - b'A' as u16)) as u32);
        }
    }
    let vk = match u.as_str() {
        "SPACE" => VK_SPACE,
        "ENTER" => VK_RETURN,
        "TAB" => VK_TAB,
        "ESCAPE" => VK_ESCAPE,
        "BACKSPACE" => VK_BACK,
        "DELETE" => VK_DELETE,
        "INSERT" => VK_INSERT,
        "HOME" => VK_HOME,
        "END" => VK_END,
        "PAGEUP" => VK_PRIOR,
        "PAGEDOWN" => VK_NEXT,
        "UP" => VK_UP,
        "DOWN" => VK_DOWN,
        "LEFT" => VK_LEFT,
        "RIGHT" => VK_RIGHT,
        _ => return None,
    };
    Some(vk.0 as u32)
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn vk_key_name(vk: u32) -> Option<String> {
    if (0x70..=0x87).contains(&vk) {
        return Some(format!("F{}", vk - 0x70 + 1));
    }
    if (0x60..=0x69).contains(&vk) {
        return Some(format!("NumPad{}", vk - 0x60));
    }
    if (0x30..=0x39).contains(&vk) {
        return Some(format!("{}", vk - 0x30));
    }
    if (0x41..=0x5a).contains(&vk) {
        return Some(format!("{}", char::from_u32(vk).unwrap_or('?')));
    }
    if vk == VK_SPACE.0 as u32 {
        return Some("Space".to_string());
    }
    if vk == VK_RETURN.0 as u32 {
        return Some("Enter".to_string());
    }
    if vk == VK_TAB.0 as u32 {
        return Some("Tab".to_string());
    }
    if vk == VK_ESCAPE.0 as u32 {
        return Some("Escape".to_string());
    }
    if vk == VK_BACK.0 as u32 {
        return Some("Backspace".to_string());
    }
    if vk == VK_DELETE.0 as u32 {
        return Some("Delete".to_string());
    }
    if vk == VK_INSERT.0 as u32 {
        return Some("Insert".to_string());
    }
    if vk == VK_HOME.0 as u32 {
        return Some("Home".to_string());
    }
    if vk == VK_END.0 as u32 {
        return Some("End".to_string());
    }
    if vk == VK_PRIOR.0 as u32 {
        return Some("PageUp".to_string());
    }
    if vk == VK_NEXT.0 as u32 {
        return Some("PageDown".to_string());
    }
    if vk == VK_UP.0 as u32 {
        return Some("Up".to_string());
    }
    if vk == VK_DOWN.0 as u32 {
        return Some("Down".to_string());
    }
    if vk == VK_LEFT.0 as u32 {
        return Some("Left".to_string());
    }
    if vk == VK_RIGHT.0 as u32 {
        return Some("Right".to_string());
    }
    if (1..=255).contains(&vk) {
        return Some(format!("Vk{vk}"));
    }
    None
}

fn current_modifier_mask() -> u32 {
    let mut m = 0u32;
    if unsafe { (GetAsyncKeyState(VK_LSHIFT.0 as i32) as u16 & 0x8000) != 0 }
        || unsafe { (GetAsyncKeyState(VK_RSHIFT.0 as i32) as u16 & 0x8000) != 0 }
        || unsafe { (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 }
    {
        m |= MOD_SHIFT.0;
    }
    if unsafe { (GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000) != 0 }
        || unsafe { (GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000) != 0 }
        || unsafe { (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 }
    {
        m |= MOD_CONTROL.0;
    }
    if unsafe { (GetAsyncKeyState(VK_LMENU.0 as i32) as u16 & 0x8000) != 0 }
        || unsafe { (GetAsyncKeyState(VK_RMENU.0 as i32) as u16 & 0x8000) != 0 }
        || unsafe { (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0 }
    {
        m |= MOD_ALT.0;
    }
    if unsafe { (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0 }
        || unsafe { (GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0 }
    {
        m |= MOD_WIN.0;
    }
    m
}

fn is_modifier_vk(vk: u32) -> bool {
    matches!(
        vk,
        _ if vk == VK_SHIFT.0 as u32
            || vk == VK_LSHIFT.0 as u32
            || vk == VK_RSHIFT.0 as u32
            || vk == VK_CONTROL.0 as u32
            || vk == VK_LCONTROL.0 as u32
            || vk == VK_RCONTROL.0 as u32
            || vk == VK_MENU.0 as u32
            || vk == VK_LMENU.0 as u32
            || vk == VK_RMENU.0 as u32
            || vk == VK_LWIN.0 as u32
            || vk == VK_RWIN.0 as u32
    )
}

fn build_normalized_hotkey(vk: u32) -> Option<String> {
    let mods = current_modifier_mask();
    let key = vk_key_name(vk)?;
    let mut raw = String::new();
    if mods & MOD_CONTROL.0 != 0 {
        raw.push_str("Ctrl+");
    }
    if mods & MOD_ALT.0 != 0 {
        raw.push_str("Alt+");
    }
    if mods & MOD_SHIFT.0 != 0 {
        raw.push_str("Shift+");
    }
    if mods & MOD_WIN.0 != 0 {
        raw.push_str("Win+");
    }
    raw.push_str(&key);
    crate::hotkeys::HotkeyService::default()
        .validate_hotkey(&raw)
        .ok()
}

extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    LL_HOOK_ENTRY_COUNT.fetch_add(1, Ordering::Relaxed);
    if n_code >= 0 {
        LL_HOOK_NONNEG_CALLS.fetch_add(1, Ordering::Relaxed);
    } else if diag::enabled()
        && !LL_HOOK_FIRST_NCODE_LOGGED.swap(true, Ordering::SeqCst)
    {
        diag::trace(
            "hotkeys",
            &format!(
                "hook_proc: first n_code<0 sample n_code={n_code} (HC_ACTION=0; negative means pass-through only)",
            ),
        );
    }
    let pass = || unsafe {
        let raw = LL_KEYBOARD_HOOK_HANDLE.load(Ordering::Relaxed);
        let hh = if raw == 0 {
            None
        } else {
            Some(HHOOK(raw as *mut _))
        };
        CallNextHookEx(hh, n_code, wparam, lparam)
    };
    if n_code < 0 {
        return pass();
    }
    let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    // WebView-directed input is often LLKHF_INJECTED; still observe keys while capturing a hotkey.
    let capture_active = CAPTURE.lock().map(|g| g.active).unwrap_or(false);
    if info.flags.contains(LLKHF_INJECTED) && !capture_active {
        return pass();
    }
    let vk = info.vkCode;
    let key_up = info.flags.contains(LLKHF_UP)
        || wparam.0 == WM_KEYUP as usize
        || wparam.0 == WM_SYSKEYUP as usize;
    let key_down =
        wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;

    if let Some(disp) = try_dispatch_hotkey_capture(
        vk,
        key_up,
        key_down,
        Some(info.scanCode),
        "capture_ll",
    ) {
        return match disp {
            HotkeyCaptureDisposition::Swallow => LRESULT(1),
            HotkeyCaptureDisposition::PassThrough => pass(),
        };
    }

    if key_up {
        return pass();
    }
    if !key_down {
        return pass();
    }

    pass()
}
