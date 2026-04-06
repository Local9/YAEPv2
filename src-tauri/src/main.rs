// Hide the console window on Windows release builds.
// Diagnostics still persist to diagnostics.log when enabled.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    yaep_rust_lib::run();
}
