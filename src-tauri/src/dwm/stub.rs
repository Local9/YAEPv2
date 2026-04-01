//! No-op DWM service when `windows` is not linked (Linux CI / non-Windows targets).

use std::sync::Arc;

use tauri::AppHandle;

use crate::db::DbService;

#[derive(Clone, Default)]
pub struct DwmService;

impl DwmService {
    pub fn sync_thumbnail_graph(&self) {}

    pub fn set_app_handle(&self, _app: AppHandle) {}

    pub fn set_db(&self, _db: Arc<DbService>) {}

    pub fn register_runtime_thumbnail(&self, _pid: u32, _source_hwnd: isize, _title: &str) {}

    pub fn unregister_runtime_thumbnail(&self, _pid: u32) {}

    pub fn set_focused_thumbnail(&self, _focused_pid: Option<u32>) {}

    pub fn ensure_missing_runtime_overlays(&self, _max_to_create: usize) {}

    pub fn snapshot_thumbnail_overlay_state(
        &self,
        _overlay_id: &str,
    ) -> Option<crate::thumbnail_webview_overlay::ThumbnailOverlayStatePayload> {
        None
    }
}
