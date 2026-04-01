//! DWM-backed thumbnail hosts (Windows only). Non-Windows builds use a no-op stub so `cargo check` works on CI.

#[cfg(target_os = "windows")]
mod imp;
#[cfg(target_os = "windows")]
pub use imp::*;

#[cfg(not(target_os = "windows"))]
mod stub;
#[cfg(not(target_os = "windows"))]
pub use stub::*;
