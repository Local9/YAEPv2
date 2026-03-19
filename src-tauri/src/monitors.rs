#[cfg(target_os = "windows")]
mod win {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
    };
    use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;

    use crate::models::MonitorInfoDto;

    struct EnumCtx {
        list: Vec<MonitorInfoDto>,
        next_index: i32,
    }

    unsafe extern "system" fn monitor_enum_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _lprc: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut EnumCtx);
        let mut mx = MONITORINFOEXW::default();
        mx.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if !GetMonitorInfoW(hmon, &mut mx.monitorInfo).as_bool() {
            return BOOL(1);
        }
        let end = mx
            .szDevice
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(mx.szDevice.len());
        let name = String::from_utf16_lossy(&mx.szDevice[..end]);
        let is_primary = (mx.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;
        let m = mx.monitorInfo.rcMonitor;
        let w = mx.monitorInfo.rcWork;
        ctx.list.push(MonitorInfoDto {
            index: ctx.next_index,
            name,
            left: m.left,
            top: m.top,
            right: m.right,
            bottom: m.bottom,
            work_left: w.left,
            work_top: w.top,
            work_right: w.right,
            work_bottom: w.bottom,
            is_primary,
            hardware_id: String::new(),
        });
        ctx.next_index += 1;
        BOOL(1)
    }

    pub fn list_monitors() -> Vec<MonitorInfoDto> {
        let mut ctx = EnumCtx {
            list: Vec::new(),
            next_index: 0,
        };
        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(monitor_enum_proc),
                LPARAM(&mut ctx as *mut EnumCtx as isize),
            );
        }
        ctx.list
    }

    pub fn work_area_offset(monitor_index: i64) -> Option<(i32, i32)> {
        let list = list_monitors();
        let idx = usize::try_from(monitor_index).ok()?;
        let m = list.get(idx)?;
        Some((m.work_left, m.work_top))
    }

    pub fn clamp_rect_to_monitor_work_area(
        monitor_index: i64,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> Option<(i32, i32)> {
        let list = list_monitors();
        let idx = usize::try_from(monitor_index).ok()?;
        let m = list.get(idx)?;
        let mut nx = x;
        let mut ny = y;
        if nx + w > m.work_right {
            nx = m.work_right - w;
        }
        if ny + h > m.work_bottom {
            ny = m.work_bottom - h;
        }
        if nx < m.work_left {
            nx = m.work_left;
        }
        if ny < m.work_top {
            ny = m.work_top;
        }
        Some((nx, ny))
    }
}

#[cfg(target_os = "windows")]
pub use win::*;

#[cfg(not(target_os = "windows"))]
use crate::models::MonitorInfoDto;

#[cfg(not(target_os = "windows"))]
pub fn list_monitors() -> Vec<MonitorInfoDto> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn work_area_offset(_monitor_index: i64) -> Option<(i32, i32)> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn clamp_rect_to_monitor_work_area(
    _monitor_index: i64,
    x: i32,
    y: i32,
    _w: i32,
    _h: i32,
) -> Option<(i32, i32)> {
    Some((x, y))
}
