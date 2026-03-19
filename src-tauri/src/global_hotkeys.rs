//! Win32 `RegisterHotKey` on a dedicated thread + `WM_HOTKEY` dispatch.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL,
    MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, VK_0, VK_A, VK_BACK, VK_DELETE, VK_DOWN, VK_END,
    VK_ESCAPE, VK_F1, VK_HOME, VK_INSERT, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_LEFT,
    VK_NEXT, VK_NUMPAD0, VK_PRIOR, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_RETURN, VK_RIGHT,
    VK_SHIFT, VK_CONTROL, VK_MENU, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT,
    LLKHF_INJECTED, LLKHF_UP, RegisterClassW, SetWindowsHookExW, TranslateMessage, HHOOK, HWND_MESSAGE,
    MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN, WINDOW_EX_STYLE, WINDOW_STYLE, WM_HOTKEY, WNDCLASSW,
};

use windows::core::PCWSTR;

use crate::diag;
use crate::AppState;

static REFRESH_TX: Mutex<Option<mpsc::Sender<()>>> = Mutex::new(None);
static THREAD_STARTED: AtomicBool = AtomicBool::new(false);

/// Hook handle as a pointer address (`HHOOK` is not `Send` in windows-rs).
static HOTKEY_HOOK_ADDR: Mutex<Option<usize>> = Mutex::new(None);
static SUPPRESS_COMBOS: OnceLock<Mutex<Vec<(u32, u32)>>> = OnceLock::new();

struct CaptureState {
    active: bool,
    capture_type: String,
    target_id: Option<i64>,
}

static CAPTURE: Mutex<CaptureState> = Mutex::new(CaptureState {
    active: false,
    capture_type: String::new(),
    target_id: None,
});

fn suppress_combos() -> &'static Mutex<Vec<(u32, u32)>> {
    SUPPRESS_COMBOS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn begin_hotkey_capture(capture_type: String, target_id: Option<i64>) {
    if let Ok(mut g) = CAPTURE.lock() {
        g.active = true;
        g.capture_type = capture_type;
        g.target_id = target_id;
    }
}

pub fn end_hotkey_capture() {
    if let Ok(mut g) = CAPTURE.lock() {
        g.active = false;
        g.capture_type.clear();
        g.target_id = None;
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
}

pub fn spawn_thread(app: AppHandle) {
    if THREAD_STARTED.swap(true, Ordering::SeqCst) {
        diag::trace("hotkeys", "spawn_thread: already started");
        return;
    }
    diag::trace("hotkeys", "spawn_thread: starting message loop thread");
    let (tx, rx) = mpsc::channel();
    *REFRESH_TX.lock().expect("refresh tx lock") = Some(tx);
    std::thread::spawn(move || run_message_loop(app, rx));
}

fn hotkey_class_name() -> &'static Vec<u16> {
    static NAME: OnceLock<Vec<u16>> = OnceLock::new();
    NAME.get_or_init(|| to_wide("YAEP_GlobalHotkeyHost"))
}

fn run_message_loop(app: AppHandle, refresh_rx: mpsc::Receiver<()>) {
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
    diag::trace(
        "hotkeys",
        &format!("message window ok hwnd=0x{:x}", hwnd.0 as isize),
    );

    unsafe {
        let hmod = GetModuleHandleW(None).unwrap_or_default();
        if let Ok(h) = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            Some(hmod.into()),
            0,
        ) {
            *HOTKEY_HOOK_ADDR.lock().expect("hook lock") = Some(h.0 as usize);
            diag::trace("hotkeys", "WH_KEYBOARD_LL hook installed");
        } else {
            diag::trace("hotkeys", "SetWindowsHookExW(WH_KEYBOARD_LL) failed");
        }
    }

    let mut registered: Vec<i32> = Vec::new();
    refresh_registrations(hwnd, &app, &mut registered);
    diag::trace("hotkeys", "entering GetMessageW loop");

    loop {
        while let Ok(()) = refresh_rx.try_recv() {
            refresh_registrations(hwnd, &app, &mut registered);
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

extern "system" fn hotkey_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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
                let _ = crate::cycle_client_group_internal(&*state, group_id, direction);
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

fn refresh_registrations(hwnd: HWND, app: &AppHandle, registered: &mut Vec<i32>) {
    for id in registered.drain(..) {
        unsafe {
            let _ = UnregisterHotKey(Some(hwnd), id);
        }
    }
    let mut table = id_table().lock().expect("id table lock poisoned");
    table.clear();
    suppress_combos().lock().expect("suppress lock").clear();

    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    let mut next_id = 1i32;
    let mut push = |table: &mut Vec<(i32, HotkeyAction)>, id: &mut i32, hk: &str, action: HotkeyAction| {
        if hk.trim().is_empty() {
            return;
        }
        let Some((mods, vk)) = parse_hotkey(hk) else {
            return;
        };
        unsafe {
            if RegisterHotKey(Some(hwnd), *id, mods, vk).is_err() {
                return;
            }
        }
        let mask = mods.0 & !MOD_NOREPEAT.0;
        suppress_combos()
            .lock()
            .expect("suppress lock")
            .push((mask, vk));
        table.push((*id, action));
        registered.push(*id);
        *id += 1;
    };

    if let Ok(profiles) = state.db.get_profiles() {
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

    if let Some(pid) = state.db.active_profile_id() {
        if let Ok(groups) = state.db.get_client_groups(pid) {
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

    if let Ok(links) = state.db.get_mumble_links() {
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
    let pass = || unsafe {
        CallNextHookEx(Some(HHOOK(std::ptr::null_mut())), n_code, wparam, lparam)
    };
    if n_code < 0 {
        return pass();
    }
    let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
    if info.flags.contains(LLKHF_INJECTED) {
        return pass();
    }
    if info.flags.contains(LLKHF_UP) {
        return pass();
    }
    if wparam.0 != WM_KEYDOWN as usize && wparam.0 != WM_SYSKEYDOWN as usize {
        return pass();
    }
    let vk = info.vkCode;

    if let Ok(mut g) = CAPTURE.lock() {
        if g.active {
            if is_modifier_vk(vk) {
                return pass();
            }
            let payload = build_normalized_hotkey(vk).map(|value| HotkeyCapturedPayload {
                value,
                capture_type: g.capture_type.clone(),
                target_id: g.target_id,
            });
            g.active = false;
            g.capture_type.clear();
            g.target_id = None;
            let app_for_emit = HOTKEY_APP_HANDLE.get().cloned();
            drop(g);
            if let (Some(app_ref), Some(payload)) = (app_for_emit, payload) {
                let app_emit = app_ref.clone();
                let _ = app_ref.run_on_main_thread(move || {
                    let _ = app_emit.emit("hotkeyCaptured", payload);
                });
            }
            return LRESULT(1);
        }
    }

    let cur = current_modifier_mask();
    if let Ok(combos) = suppress_combos().lock() {
        for (m, k) in combos.iter() {
            if *m == cur {
                if *m == 0 {
                    if vk == *k {
                        return LRESULT(1);
                    }
                } else if !is_modifier_vk(vk) {
                    return LRESULT(1);
                }
            }
        }
    }

    pass()
}
