#[derive(Debug, Clone)]
pub struct WindowSnapshot {
    pub pid: u32,
    pub hwnd: isize,
    pub title: String,
}

use crate::dwm::DwmService;

#[derive(Default)]
pub struct WindowService;

impl WindowService {
    pub fn apply_grid_layout(&self, dwm: &DwmService) {
        dwm.sync_thumbnail_graph();
    }

    #[cfg(target_os = "windows")]
    pub fn activate_window_by_pid(&self, target_pid: u32) -> Result<(), String> {
        use std::ffi::c_void;
        use windows::core::BOOL;
        use windows::Win32::Foundation::{HWND, LPARAM};
        use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
        use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
            SetForegroundWindow, ShowWindow, SW_RESTORE,
        };

        #[derive(Default)]
        struct ActivationCtx {
            target_pid: u32,
            hwnd: Option<HWND>,
        }

        unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let ctx = &mut *(lparam.0 as *mut ActivationCtx);
            if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
                return BOOL(1);
            }
            let mut pid: u32 = 0;
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
            }
            if pid != ctx.target_pid {
                return BOOL(1);
            }
            ctx.hwnd = Some(hwnd);
            BOOL(0)
        }

        let mut ctx = ActivationCtx {
            target_pid,
            hwnd: None,
        };
        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut ctx as *mut ActivationCtx as *mut c_void as isize),
            );
        }
        let Some(hwnd) = ctx.hwnd else {
            return Err("No visible window found for target PID".to_string());
        };

        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
            let fg = GetForegroundWindow();
            if !fg.is_invalid() {
                let mut fg_tid = 0u32;
                GetWindowThreadProcessId(fg, Some(&mut fg_tid));
                let cur_tid = GetCurrentThreadId();
                let _ = AttachThreadInput(cur_tid, fg_tid, true);
                let _ = SetForegroundWindow(hwnd);
                let _ = SetFocus(Some(hwnd));
                let _ = AttachThreadInput(cur_tid, fg_tid, false);
            } else {
                let _ = SetForegroundWindow(hwnd);
                let _ = SetFocus(Some(hwnd));
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn activate_window_by_pid(&self, _target_pid: u32) -> Result<(), String> {
        Err("Window activation is only supported on Windows".to_string())
    }

    pub fn activate_window_by_title(&self, title: &str) -> Result<(), String> {
        let target = self
            .enumerate_windows()
            .into_iter()
            .find(|w| w.title == title)
            .ok_or_else(|| "No window found for title".to_string())?;
        self.activate_window_by_pid(target.pid)
    }

    #[cfg(target_os = "windows")]
    pub fn enumerate_windows(&self) -> Vec<WindowSnapshot> {
        use windows::core::BOOL;
        use windows::Win32::Foundation::{HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        };

        unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let windows = &mut *(lparam.0 as *mut Vec<WindowSnapshot>);
            if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
                return BOOL(1);
            }

            let title_length = unsafe { GetWindowTextLengthW(hwnd) };
            if title_length <= 0 {
                return BOOL(1);
            }

            let mut title_buf = vec![0u16; (title_length + 1) as usize];
            let read_len = unsafe { GetWindowTextW(hwnd, &mut title_buf) };
            if read_len <= 0 {
                return BOOL(1);
            }

            let title = String::from_utf16_lossy(&title_buf[..read_len as usize])
                .trim()
                .to_string();
            if title.is_empty() {
                return BOOL(1);
            }

            let mut pid: u32 = 0;
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
            }
            if pid == 0 {
                return BOOL(1);
            }

            windows.push(WindowSnapshot {
                pid,
                hwnd: hwnd.0 as isize,
                title,
            });
            BOOL(1)
        }

        let mut windows = Vec::new();
        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<WindowSnapshot> as isize),
            );
        }
        windows
    }

    #[cfg(not(target_os = "windows"))]
    pub fn enumerate_windows(&self) -> Vec<WindowSnapshot> {
        Vec::new()
    }

    #[cfg(target_os = "windows")]
    pub fn foreground_window_pid(&self) -> Option<u32> {
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        if pid == 0 {
            None
        } else {
            Some(pid)
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn foreground_window_pid(&self) -> Option<u32> {
        None
    }
}
