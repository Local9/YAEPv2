use crate::models::Profile;

#[derive(Default)]
pub struct DbService;

impl DbService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_profiles(&self) -> Vec<Profile> {
        vec![Profile {
            id: 1,
            name: "Default".to_string(),
            deleted_at: None,
            is_active: true,
            switch_hotkey: String::new(),
        }]
    }

    pub fn active_profile_id(&self) -> Option<i64> {
        Some(1)
    }
}
