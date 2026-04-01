#[cfg(target_os = "windows")]
mod windows_guard {
    use std::sync::OnceLock;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;

    /// Raw mutex handle value; `HANDLE` is not `Send`/`Sync` for `OnceLock`.
    static INSTANCE_MUTEX: OnceLock<isize> = OnceLock::new();

    pub fn ensure_single_instance() -> Result<(), String> {
        let name = to_wide("YAEP_RUST_SINGLE_INSTANCE_MUTEX");
        let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }
            .map_err(|e| format!("failed to create single-instance mutex: {e}"))?;

        let last_error = unsafe { GetLastError() };
        if last_error == ERROR_ALREADY_EXISTS {
            return Err("another YAEP instance is already running".to_string());
        }

        let _ = INSTANCE_MUTEX.set(handle.0 as isize);
        Ok(())
    }

    fn to_wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(target_os = "windows")]
pub fn ensure_single_instance() -> Result<(), String> {
    windows_guard::ensure_single_instance()
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_single_instance() -> Result<(), String> {
    Ok(())
}
