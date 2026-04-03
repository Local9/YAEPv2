//! Records the thread that runs `App::setup` so DWM and other code can avoid
//! `run_on_main_thread` + blocking `recv` when already on the GUI thread (deadlocks the webview).

use std::sync::OnceLock;

static GUI_THREAD_ID: OnceLock<std::thread::ThreadId> = OnceLock::new();

/// Call once from `App::setup` before any commands that may run on this thread.
pub fn register_gui_thread() {
    let _ = GUI_THREAD_ID.set(std::thread::current().id());
}

pub fn is_gui_thread() -> bool {
    GUI_THREAD_ID.get() == Some(&std::thread::current().id())
}
