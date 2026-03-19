#[derive(Default)]
pub struct HotkeyService;

impl HotkeyService {
    pub fn capture_start(&self, _capture_type: String, _target_id: Option<i64>) {}
    pub fn capture_stop(&self) {}
}
