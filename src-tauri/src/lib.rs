mod db;
mod dwm;
mod error;
mod eve_profile_tools;
mod hotkeys;
mod models;
mod thumbnail_service;
mod windows;

use std::sync::Arc;
use tauri::State;

use crate::db::DbService;
use crate::models::{HealthSnapshot, Profile};

pub struct AppState {
    db: Arc<DbService>,
}

impl AppState {
    fn new() -> Self {
        Self {
            db: Arc::new(DbService::new()),
        }
    }
}

#[tauri::command]
fn health(state: State<'_, AppState>) -> Result<HealthSnapshot, String> {
    Ok(HealthSnapshot {
        app: "yaep-rust",
        backend_ready: true,
        active_profile_id: state.db.active_profile_id(),
    })
}

#[tauri::command]
fn get_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    Ok(state.db.get_profiles())
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![health, get_profiles])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
