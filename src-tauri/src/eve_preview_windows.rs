//! EVE client windows eligible for thumbnails (process allow-list + title rules).
//! Shared by the thumbnail monitor and settings-import reapply.

use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::db::DbService;
use crate::windows::{WindowService, WindowSnapshot};

const BASE_EVE_TITLE: &str = "EVE";
const CHARACTER_TITLE_PREFIX: &str = "EVE - ";

pub fn normalize_process_name(name: &str) -> String {
    name.trim().trim_end_matches(".exe").to_lowercase()
}

fn filter_preview_windows(
    windows: Vec<WindowSnapshot>,
    sys: &System,
    target_processes: &[String],
) -> Vec<WindowSnapshot> {
    windows
        .into_iter()
        .filter(|window| {
            let pid = Pid::from_u32(window.pid);
            let Some(process) = sys.process(pid) else {
                return false;
            };
            let process_name = normalize_process_name(process.name().to_string_lossy().as_ref());
            if !target_processes.contains(&process_name) {
                return false;
            }

            if window.title.trim() == BASE_EVE_TITLE {
                return false;
            }
            window.title.starts_with(CHARACTER_TITLE_PREFIX)
        })
        .collect()
}

/// Returns [`None`] if the monitor should skip this cycle (no active profile or DB read error).
/// Returns [`Some`] (possibly empty) when the preview list should replace the prior snapshot.
pub fn try_list_preview_eve_windows(
    db: &DbService,
    windows: &WindowService,
    sys: &mut System,
) -> Option<Vec<WindowSnapshot>> {
    let profile_id = db.active_profile_id()?;
    let process_names = db.get_processes_to_preview(profile_id).ok()?;
    let target_processes: Vec<String> = process_names
        .into_iter()
        .map(|p| normalize_process_name(&p))
        .collect();

    sys.refresh_processes(ProcessesToUpdate::All, true);
    let candidates = windows.enumerate_windows();
    Some(filter_preview_windows(candidates, sys, &target_processes))
}
