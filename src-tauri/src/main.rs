// Keep a diagnostics console visible on Windows release builds so stdout/stderr
// and panic output are available for troubleshooting.
#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

fn main() {
    yaep_rust_lib::run();
}
