//! No-op DWM service when `windows` is not linked (Linux CI / non-Windows targets).

use std::sync::Arc;

use tauri::AppHandle;

use crate::db::DbService;

#[derive(Clone, Default)]
pub struct DwmService;

impl DwmService {
    pub fn sync_thumbnail_graph(&self) {}

    pub fn request_thumbnail_layout_sync(&self) {}

    pub(crate) fn take_pending_thumbnail_layout_sync(&self) -> bool {
        false
    }

    pub fn set_app_handle(&self, _app: AppHandle) {}

    pub fn set_db(&self, _db: Arc<DbService>) {}

    pub fn register_runtime_thumbnail(&self, _pid: u32, _source_hwnd: isize, _title: &str) {}

    pub fn register_runtime_thumbnail_no_persist(&self, _pid: u32, _source_hwnd: isize, _title: &str) {}

    pub fn unregister_runtime_thumbnail(&self, _pid: u32) {}

    pub fn prune_runtime_thumbnails_not_matching(&self, _keep_pids: &[u32]) {}

    pub fn set_focused_thumbnail(&self, _focused_pid: Option<u32>) {}

    pub fn ensure_missing_runtime_overlays(&self, _max_to_create: usize) {}

    pub fn snapshot_thumbnail_overlay_state(
        &self,
        _overlay_id: &str,
    ) -> Option<crate::thumbnail_webview_overlay::ThumbnailOverlayStatePayload> {
        None
    }
}
