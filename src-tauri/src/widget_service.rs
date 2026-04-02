use std::sync::Mutex;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IntelWidgetLine {
    pub timestamp: String,
    pub channel_name: String,
    pub message: String,
    pub background_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WidgetSnapshot {
    pub fleet_motd: String,
    pub intel_lines: Vec<IntelWidgetLine>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct WidgetUpdateEvent {
    snapshot: WidgetSnapshot,
}

#[derive(Default)]
pub struct WidgetService {
    state: Mutex<WidgetSnapshot>,
}

impl WidgetService {
    pub fn snapshot(&self) -> WidgetSnapshot {
        self.state.lock().unwrap().clone()
    }

    pub fn ingest_fleet_motd(&self, app: &AppHandle, motd: String) {
        let snapshot = {
            let mut guard = self.state.lock().unwrap();
            guard.fleet_motd = motd;
            guard.clone()
        };
        let _ = app.emit("widget:update", WidgetUpdateEvent { snapshot });
    }

    pub fn ingest_intel_line(
        &self,
        app: &AppHandle,
        timestamp: String,
        channel_name: String,
        message: String,
        background_color: String,
    ) {
        let snapshot = {
            let mut guard = self.state.lock().unwrap();
            guard.intel_lines.push(IntelWidgetLine {
                timestamp,
                channel_name,
                message,
                background_color,
            });
            if guard.intel_lines.len() > 50 {
                let keep_from = guard.intel_lines.len() - 50;
                guard.intel_lines = guard.intel_lines.split_off(keep_from);
            }
            guard.clone()
        };
        let _ = app.emit("widget:update", WidgetUpdateEvent { snapshot });
    }

    pub fn refresh_intel_channel_colors(
        &self,
        app: &AppHandle,
        color_by_channel: &HashMap<String, String>,
    ) {
        let snapshot = {
            let mut guard = self.state.lock().unwrap();
            for line in &mut guard.intel_lines {
                if let Some(color) = color_by_channel.get(&line.channel_name) {
                    line.background_color = color.clone();
                }
            }
            guard.clone()
        };
        let _ = app.emit("widget:update", WidgetUpdateEvent { snapshot });
    }
}
