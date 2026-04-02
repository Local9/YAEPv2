use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::db::DbService;
use crate::models::{EveChatChannel, ThumbnailSetting};
use crate::widget_service::WidgetService;

#[derive(Default)]
pub struct EveChatLogService {
    stop_tx: Mutex<Option<Sender<()>>>,
}

#[derive(Default)]
struct FileCursor {
    path: PathBuf,
    offset: u64,
}

#[derive(Clone)]
struct ChannelTarget {
    channel_type: String,
    channel_name: String,
    background_color: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ThumbnailStatusEvent {
    window_title: Option<String>,
    character_id: Option<i64>,
    listener_name: Option<String>,
    system: Option<String>,
    is_cloaked: Option<bool>,
    decloak_flash: Option<bool>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FleetMotdEvent {
    motd: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntelLineEvent {
    timestamp: String,
    channel_name: String,
    message: String,
    background_color: String,
}

#[derive(Default)]
struct Routing {
    by_character_id: HashMap<i64, String>,
    by_listener_name: HashMap<String, String>,
}

impl EveChatLogService {
    pub fn start(
        &self,
        app: AppHandle,
        db: std::sync::Arc<DbService>,
        widget_service: std::sync::Arc<WidgetService>,
    ) {
        self.stop();
        let (tx, rx) = mpsc::channel::<()>();
        *self.stop_tx.lock().unwrap() = Some(tx);
        thread::spawn(move || {
            let mut cursors: HashMap<String, FileCursor> = HashMap::new();
            let mut last_motd = String::new();
            let mut intel_seeded = false;
            loop {
                match rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {}
                }
                let Some(profile_id) = db.active_profile_id() else {
                    thread::sleep(Duration::from_millis(1500));
                    continue;
                };
                let settings = match db.get_eve_log_settings(profile_id) {
                    Ok(v) => v,
                    Err(_) => {
                        thread::sleep(Duration::from_millis(1500));
                        continue;
                    }
                };
                let channels = db.list_eve_chat_channels(profile_id).unwrap_or_default();
                let thumbnails = db.get_thumbnail_settings(profile_id).unwrap_or_default();
                let routing = build_routing(&thumbnails);
                let targets = build_channel_targets(channels);
                process_chat_logs(
                    &app,
                    &settings.chat_logs_path,
                    &targets,
                    &routing,
                    &mut cursors,
                    &mut last_motd,
                    widget_service.as_ref(),
                );
                if !intel_seeded {
                    seed_recent_intel_messages(
                        &app,
                        &settings.chat_logs_path,
                        &targets,
                        widget_service.as_ref(),
                    );
                    intel_seeded = true;
                }
                process_game_logs(
                    &app,
                    &settings.game_logs_path,
                    &routing,
                    &mut cursors,
                );
                thread::sleep(Duration::from_millis(1500));
            }
        });
    }

    pub fn stop(&self) {
        if let Some(tx) = self.stop_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
    }
}

fn build_channel_targets(channels: Vec<EveChatChannel>) -> Vec<ChannelTarget> {
    let mut out = vec![
        ChannelTarget {
            channel_type: "Fleet".to_string(),
            channel_name: "Fleet".to_string(),
            background_color: "#1f2937".to_string(),
        },
        ChannelTarget {
            channel_type: "Local".to_string(),
            channel_name: "Local".to_string(),
            background_color: "#1f2937".to_string(),
        },
    ];
    for c in channels {
        out.push(ChannelTarget {
            channel_type: c.channel_type,
            channel_name: c.channel_name,
            background_color: c.background_color,
        });
    }
    out
}

fn build_routing(thumbnails: &[ThumbnailSetting]) -> Routing {
    let mut routing = Routing::default();
    for setting in thumbnails {
        if let Some(character_id) = setting.character_id {
            routing
                .by_character_id
                .insert(character_id, setting.window_title.clone());
        }
        let clean = setting
            .window_title
            .strip_prefix("EVE - ")
            .unwrap_or(&setting.window_title)
            .trim()
            .to_ascii_lowercase();
        if !clean.is_empty() {
            routing
                .by_listener_name
                .entry(clean)
                .or_insert_with(|| setting.window_title.clone());
        }
    }
    routing
}

fn process_chat_logs(
    app: &AppHandle,
    dir: &str,
    channels: &[ChannelTarget],
    routing: &Routing,
    cursors: &mut HashMap<String, FileCursor>,
    last_motd: &mut String,
    widget_service: &WidgetService,
) {
    for target in channels {
        let chat_sources: Vec<(PathBuf, Option<i64>)> = if target.channel_type == "Local" {
            latest_local_chat_files_by_character(dir, &target.channel_name)
        } else {
            latest_chat_file(dir, &target.channel_name)
                .map(|path| {
                    vec![(
                        path.clone(),
                        parse_trailing_character_id(path.file_name().and_then(|n| n.to_str())),
                    )]
                })
                .unwrap_or_default()
        };
        for (path, character_id) in chat_sources {
            let key = format!(
                "chat:{}:{}:{}",
                target.channel_type,
                target.channel_name,
                character_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string())
            );
            let lines = read_new_lines(cursors, &key, &path);
            let mut chat_listener_name: Option<String> = None;
            for line in lines {
                let clean = sanitize_rich_text(&line);
                if target.channel_type == "Local" {
                    if let Some(system) = parse_local_system_marker(&clean) {
                        let payload = ThumbnailStatusEvent {
                            window_title: resolve_window_title(
                                routing,
                                character_id,
                                chat_listener_name.as_deref(),
                            ),
                            character_id,
                            listener_name: chat_listener_name.clone(),
                            system: Some(system.clone()),
                            is_cloaked: None,
                            decloak_flash: None,
                        };
                        let _ = app.emit("eve-chat:local-update", &system);
                        let _ = app.emit("eve-thumbnail-status", payload);
                        continue;
                    }
                }
                if should_skip_chat_header_line(&clean) {
                    continue;
                }
                if let Some(name) = parse_listener_name(&clean) {
                    chat_listener_name = Some(name);
                    continue;
                }
                if target.channel_type == "Fleet" && clean.to_ascii_lowercase().contains("motd") {
                    let formatted_motd = format_fleet_motd(&clean);
                    if clean != *last_motd {
                        *last_motd = clean.clone();
                        let _ = app.emit("eve-fleet:motd", FleetMotdEvent { motd: formatted_motd.clone() });
                        widget_service.ingest_fleet_motd(app, formatted_motd);
                    }
                    continue;
                }
                if target.channel_type == "Local" {
                    continue;
                }
                let event_name = if target.channel_type == "FleetBoost" || target.channel_type == "Fleet" {
                    "eve-chat:fleet-update"
                } else {
                    "eve-chat:local-update"
                };
                let event = ThumbnailStatusEvent {
                    window_title: resolve_window_title(routing, character_id, None),
                    character_id,
                    listener_name: None,
                    system: None,
                    is_cloaked: None,
                    decloak_flash: None,
                };
                if target.channel_type == "Intel" {
                    let (timestamp, message) = split_chat_timestamp_and_message(&clean);
                    if timestamp.trim().is_empty() {
                        continue;
                    }
                    if should_skip_intel_message(&message) {
                        continue;
                    }
                    let intel_line = IntelLineEvent {
                        timestamp,
                        channel_name: target.channel_name.clone(),
                        message,
                        background_color: target.background_color.clone(),
                    };
                    widget_service.ingest_intel_line(
                        app,
                        intel_line.timestamp.clone(),
                        intel_line.channel_name.clone(),
                        intel_line.message.clone(),
                        target.background_color.clone(),
                    );
                    let _ = app.emit("eve-chat:intel-line", intel_line);
                } else {
                    let _ = app.emit(event_name, &clean);
                }
                let _ = app.emit("eve-thumbnail-status", event);
            }
            if target.channel_type == "Local" {
                emit_latest_local_snapshot_from_file(app, routing, &path, character_id);
            }
        }
    }
}

fn latest_local_chat_files_by_character(dir: &str, channel_name: &str) -> Vec<(PathBuf, Option<i64>)> {
    let mut best_with_id: HashMap<i64, (String, PathBuf)> = HashMap::new();
    let mut best_without_id: Option<(String, PathBuf)> = None;
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.ends_with(".txt") {
            continue;
        }
        let Some(ts) = extract_chat_timestamp(name, channel_name) else {
            continue;
        };
        if let Some(cid) = parse_trailing_character_id(Some(name)) {
            if best_with_id
                .get(&cid)
                .is_none_or(|(best_ts, _)| ts > *best_ts)
            {
                best_with_id.insert(cid, (ts, path));
            }
        } else if best_without_id
            .as_ref()
            .is_none_or(|(best_ts, _)| ts > *best_ts)
        {
            best_without_id = Some((ts, path));
        }
    }
    let mut out: Vec<(PathBuf, Option<i64>)> = best_with_id
        .into_iter()
        .map(|(cid, (_, path))| (path, Some(cid)))
        .collect();
    if let Some((_, path)) = best_without_id {
        out.push((path, None));
    }
    out
}

fn emit_latest_local_snapshot_from_file(
    app: &AppHandle,
    routing: &Routing,
    path: &Path,
    character_id: Option<i64>,
) {
    let lines = read_recent_lines(path, 100);
    let mut latest_system: Option<String> = None;
    let mut listener_name: Option<String> = None;
    for line in lines {
        let clean = sanitize_rich_text(&line);
        if let Some(name) = parse_listener_name(&clean) {
            listener_name = Some(name);
            continue;
        }
        if let Some(system) = parse_local_system_marker(&clean) {
            latest_system = Some(system);
        }
    }
    let Some(system) = latest_system else {
        return;
    };
    let payload = ThumbnailStatusEvent {
        window_title: resolve_window_title(routing, character_id, listener_name.as_deref()),
        character_id,
        listener_name,
        system: Some(system.clone()),
        is_cloaked: None,
        decloak_flash: None,
    };
    let _ = app.emit("eve-chat:local-update", &system);
    let _ = app.emit("eve-thumbnail-status", payload);
}

fn seed_recent_intel_messages(
    app: &AppHandle,
    dir: &str,
    channels: &[ChannelTarget],
    widget_service: &WidgetService,
) {
    let mut seeded_lines: Vec<IntelLineEvent> = Vec::new();
    for target in channels {
        if target.channel_type != "Intel" {
            continue;
        }
        let Some(path) = latest_chat_file(dir, &target.channel_name) else {
            continue;
        };
        let lines = read_recent_lines(&path, 25);
        for line in lines {
            let clean = sanitize_rich_text(&line);
            if clean.is_empty() {
                continue;
            }
            if should_skip_chat_header_line(&clean) {
                continue;
            }
            let (timestamp, message) = split_chat_timestamp_and_message(&clean);
            if timestamp.trim().is_empty() {
                continue;
            }
            if should_skip_intel_message(&message) {
                continue;
            }
            seeded_lines.push(IntelLineEvent {
                timestamp,
                channel_name: target.channel_name.clone(),
                message,
                background_color: target.background_color.clone(),
            });
        }
    }
    if seeded_lines.len() > 50 {
        let keep_from = seeded_lines.len() - 50;
        seeded_lines = seeded_lines.split_off(keep_from);
    }
    for intel_line in seeded_lines {
        widget_service.ingest_intel_line(
            app,
            intel_line.timestamp.clone(),
            intel_line.channel_name.clone(),
            intel_line.message.clone(),
            intel_line.background_color.clone(),
        );
    }
}

fn process_game_logs(
    app: &AppHandle,
    dir: &str,
    routing: &Routing,
    cursors: &mut HashMap<String, FileCursor>,
) {
    let files = latest_game_files(dir);
    for (key, path, character_id) in files {
        let lines = read_new_lines(cursors, &format!("game:{key}"), &path);
        let mut listener_name: Option<String> = None;
        let mut trusted_character_id = character_id;
        for line in lines {
            let clean = sanitize_rich_text(&line);
            if let Some(name) = parse_listener_name(&clean) {
                listener_name = Some(name.clone());
                if let Some(cid) = character_id {
                    if !is_listener_consistent_with_character_id(routing, cid, &name) {
                        // Mismatch between filename character id and log header listener:
                        // avoid authoritative attribution to the wrong character id.
                        trusted_character_id = None;
                    }
                }
                continue;
            }
            if let Some(system) = parse_jump_destination(&clean) {
                let payload = ThumbnailStatusEvent {
                    window_title: resolve_window_title(
                        routing,
                        trusted_character_id,
                        listener_name.as_deref(),
                    ),
                    character_id: trusted_character_id,
                    listener_name: listener_name.clone(),
                    system: Some(system.clone()),
                    is_cloaked: None,
                    decloak_flash: None,
                };
                let _ = app.emit("eve-game:movement", &system);
                let _ = app.emit("eve-thumbnail-status", payload);
            }
            if let Some(is_cloaked) = parse_cloak_state(&clean) {
                let payload = ThumbnailStatusEvent {
                    window_title: resolve_window_title(
                        routing,
                        trusted_character_id,
                        listener_name.as_deref(),
                    ),
                    character_id: trusted_character_id,
                    listener_name: listener_name.clone(),
                    system: None,
                    is_cloaked: Some(is_cloaked),
                    decloak_flash: None,
                };
                let _ = app.emit("eve-game:cloak-state", is_cloaked);
                let _ = app.emit("eve-thumbnail-status", payload);
            }
            if is_decloak_flash_trigger(&clean) {
                let payload = ThumbnailStatusEvent {
                    window_title: resolve_window_title(
                        routing,
                        trusted_character_id,
                        listener_name.as_deref(),
                    ),
                    character_id: trusted_character_id,
                    listener_name: listener_name.clone(),
                    system: None,
                    is_cloaked: None,
                    decloak_flash: Some(true),
                };
                let _ = app.emit("eve-thumbnail-status", payload);
            }
            if clean.contains("(combat)")
                || clean.contains("Warp disruption attempt")
                || clean.contains("Warp scramble attempt")
            {
                let _ = app.emit("eve-game:combat-cue", clean.clone());
            }
        }
    }
}

fn latest_chat_file(dir: &str, channel_name: &str) -> Option<PathBuf> {
    let mut best: Option<(String, PathBuf)> = None;
    let read_dir = fs::read_dir(dir).ok()?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.ends_with(".txt") {
            continue;
        }
        if let Some(ts) = extract_chat_timestamp(name, channel_name) {
            if best.as_ref().is_none_or(|(best_ts, _)| ts > *best_ts) {
                best = Some((ts, path));
            }
        }
    }
    best.map(|(_, path)| path)
}

fn latest_game_files(dir: &str) -> Vec<(String, PathBuf, Option<i64>)> {
    let mut best_per_key: HashMap<String, (String, PathBuf, Option<i64>)> = HashMap::new();
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.ends_with(".txt") {
            continue;
        }
        let Some(ts) = extract_game_timestamp(name) else {
            continue;
        };
        let character_id = parse_trailing_character_id(Some(name));
        let key = character_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        if best_per_key
            .get(&key)
            .is_none_or(|(best_ts, _, _)| ts > *best_ts)
        {
            best_per_key.insert(key.clone(), (ts, path, character_id));
        }
    }
    best_per_key
        .into_iter()
        .map(|(k, (_, path, cid))| (k, path, cid))
        .collect()
}

fn read_new_lines(cursors: &mut HashMap<String, FileCursor>, key: &str, path: &Path) -> Vec<String> {
    let cursor = cursors.entry(key.to_string()).or_default();
    if cursor.path != path {
        cursor.path = path.to_path_buf();
        cursor.offset = 0;
    }
    let Ok(mut file) = fs::File::open(path) else {
        return Vec::new();
    };
    if file.seek(SeekFrom::Start(cursor.offset)).is_err() {
        cursor.offset = 0;
        let _ = file.seek(SeekFrom::Start(0));
    }
    let mut bytes = Vec::<u8>::new();
    if file.read_to_end(&mut bytes).is_err() {
        return Vec::new();
    }
    cursor.offset += bytes.len() as u64;
    decode_log_bytes_to_lines(&bytes)
}

fn read_recent_lines(path: &Path, max_lines: usize) -> Vec<String> {
    let Ok(file) = fs::File::open(path) else {
        return Vec::new();
    };
    let mut reader = BufReader::new(file);
    let mut bytes = Vec::<u8>::new();
    if reader.read_to_end(&mut bytes).is_err() {
        return Vec::new();
    }
    let all_lines = decode_log_bytes_to_lines(&bytes);
    if all_lines.len() <= max_lines {
        return all_lines;
    }
    all_lines[all_lines.len() - max_lines..].to_vec()
}

fn decode_log_bytes_to_lines(bytes: &[u8]) -> Vec<String> {
    if bytes.is_empty() {
        return Vec::new();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.lines().map(|line| line.to_string()).collect();
    }
    // EVE logs can be UTF-16LE on some installs; fallback decode.
    if looks_like_utf16_le(bytes) {
        let mut start = 0usize;
        if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            start = 2;
        }
        let mut units: Vec<u16> = Vec::with_capacity((bytes.len() - start) / 2);
        let mut i = start;
        while i + 1 < bytes.len() {
            units.push(u16::from_le_bytes([bytes[i], bytes[i + 1]]));
            i += 2;
        }
        let text = String::from_utf16_lossy(&units);
        return text.lines().map(|line| line.to_string()).collect();
    }
    String::from_utf8_lossy(bytes)
        .lines()
        .map(|line| line.to_string())
        .collect()
}

fn looks_like_utf16_le(bytes: &[u8]) -> bool {
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        return true;
    }
    if bytes.len() < 8 {
        return false;
    }
    let mut zero_high_bytes = 0usize;
    let mut checked = 0usize;
    let upper = bytes.len().min(128);
    let mut i = 1usize;
    while i < upper {
        if bytes[i] == 0 {
            zero_high_bytes += 1;
        }
        checked += 1;
        i += 2;
    }
    checked > 0 && (zero_high_bytes * 100 / checked) >= 30
}

fn extract_chat_timestamp(file_name: &str, channel_name: &str) -> Option<String> {
    let prefix = format!("{channel_name}_");
    if !file_name.starts_with(&prefix) {
        return None;
    }
    let tail = file_name.strip_prefix(&prefix)?.strip_suffix(".txt")?;
    let mut parts = tail.split('_');
    let date = parts.next()?;
    let time = parts.next()?;
    if date.len() != 8 || time.len() != 6 {
        return None;
    }
    Some(format!("{date}_{time}"))
}

fn extract_game_timestamp(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(".txt")?;
    let mut parts = stem.split('_');
    let date = parts.next()?;
    let time = parts.next()?;
    if date.len() != 8 || time.len() != 6 {
        return None;
    }
    Some(format!("{date}_{time}"))
}

fn parse_trailing_character_id(file_name: Option<&str>) -> Option<i64> {
    let value = file_name?.strip_suffix(".txt")?;
    let id_part = value.rsplit('_').next()?;
    id_part.parse::<i64>().ok()
}

fn parse_listener_name(line: &str) -> Option<String> {
    let marker = "Listener:";
    let idx = line.find(marker)?;
    let name = line[idx + marker.len()..].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn parse_jump_destination(line: &str) -> Option<String> {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"Jumping from (.+?) to (.+)$").unwrap());
    let caps = re.captures(line)?;
    Some(caps.get(2)?.as_str().trim().to_string())
}

fn parse_local_system_marker(line: &str) -> Option<String> {
    static RE_CHANNEL_CHANGE: OnceLock<regex::Regex> = OnceLock::new();
    static RE_LOCAL_COLON: OnceLock<regex::Regex> = OnceLock::new();
    let re_channel_change = RE_CHANNEL_CHANGE.get_or_init(|| {
        regex::Regex::new(r"(?i)(?:eve system\s*>\s*)?channel changed to local\s*[:\-]\s*(.+)$")
            .unwrap()
    });
    if let Some(caps) = re_channel_change.captures(line) {
        return Some(caps.get(1)?.as_str().trim().to_string());
    }
    let re_local_colon = RE_LOCAL_COLON
        .get_or_init(|| regex::Regex::new(r"(?i)\blocal\s*[:\-]\s*([A-Za-z0-9\- ]+)$").unwrap());
    if let Some(caps) = re_local_colon.captures(line) {
        return Some(caps.get(1)?.as_str().trim().to_string());
    }
    None
}

fn parse_cloak_state(line: &str) -> Option<bool> {
    let lower = line.to_ascii_lowercase();
    if lower.contains("while cloaked") || lower.contains("ship is cloaked") {
        return Some(true);
    }
    if lower.contains("decloak") || lower.contains("de-cloak") || lower.contains("cloaking device deactivates") {
        return Some(false);
    }
    None
}

fn is_decloak_flash_trigger(line: &str) -> bool {
    line.contains("(notify) Your cloak deactivates due to")
}

fn sanitize_rich_text(line: &str) -> String {
    static TAG_RE: OnceLock<regex::Regex> = OnceLock::new();
    static BR_RE: OnceLock<regex::Regex> = OnceLock::new();
    let br_re = BR_RE.get_or_init(|| regex::Regex::new(r"<br\s*/?>").unwrap());
    let re = TAG_RE.get_or_init(|| regex::Regex::new(r"</?(color|font|b)[^>]*>").unwrap());
    let with_newlines = br_re.replace_all(line, "\n");
    re.replace_all(line, "")
        .replace('\u{feff}', "")
        .trim()
        .to_string()
}

fn format_fleet_motd(line: &str) -> String {
    let mut text = line.replace('\u{feff}', "");
    if let Some((_, tail)) = text.split_once("Channel MOTD:") {
        text = tail.trim().to_string();
    }
    // Keep separator bars readable.
    text = text.replace("----------", "\n----------\n");
    // Known MOTD fields that should begin on their own line.
    let field_markers = [
        "Comms :",
        "Comms:",
        "ANCHOR -",
        "SHIPTYPE -",
        "LOGI CHANNEL :",
        "BOOST CHANNEL :",
        "Doctrine:",
        "Ships:",
        "Implants:",
        "Main Anchor:",
        "Backup Anchor:",
        "Logi Anchor:",
        "Forming:",
        "Fittings:",
        "Notes:",
        "Logistics >",
    ];
    for marker in field_markers {
        let needle = format!(" {marker}");
        text = text.replace(&needle, &format!("\n{marker}"));
    }
    // Final cleanup: trim each line and remove empty duplicates.
    let mut out_lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        out_lines.push(trimmed.to_string());
    }
    out_lines.join("\n")
}

fn should_skip_chat_header_line(line: &str) -> bool {
    if line.is_empty() {
        return true;
    }
    let lower = line.to_ascii_lowercase();
    lower.starts_with("listener:")
        || lower.starts_with("session started:")
        || lower.starts_with("channel changed to")
        || lower.starts_with("channel id:")
        || lower.starts_with("eve system")
}

fn should_skip_intel_message(message: &str) -> bool {
    let lower = message.trim().to_ascii_lowercase();
    lower.starts_with("eve system")
        || lower.starts_with("eve system >")
        || lower.starts_with("eve system:")
}

fn split_chat_timestamp_and_message(line: &str) -> (String, String) {
    static TIMESTAMP_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = TIMESTAMP_RE
        .get_or_init(|| regex::Regex::new(r"^\s*\[\s*([^\]]+?)\s*\]\s*(.*)$").unwrap());
    if let Some(caps) = re.captures(line) {
        let timestamp = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let message = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        return (timestamp, message);
    }
    ("".to_string(), line.trim().to_string())
}

fn resolve_window_title(
    routing: &Routing,
    character_id: Option<i64>,
    listener_name: Option<&str>,
) -> Option<String> {
    if let Some(cid) = character_id {
        if let Some(title) = routing.by_character_id.get(&cid) {
            return Some(title.clone());
        }
    }
    let Some(listener) = listener_name else {
        return None;
    };
    let key = listener.trim().to_ascii_lowercase();
    routing.by_listener_name.get(&key).cloned()
}

fn is_listener_consistent_with_character_id(routing: &Routing, character_id: i64, listener_name: &str) -> bool {
    let Some(window_title) = routing.by_character_id.get(&character_id) else {
        // No explicit title mapping for this character id yet; cannot disprove consistency.
        return true;
    };
    let expected = window_title
        .strip_prefix("EVE - ")
        .unwrap_or(window_title)
        .trim()
        .to_ascii_lowercase();
    let actual = listener_name.trim().to_ascii_lowercase();
    expected == actual
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chat_timestamp_with_channel_prefix() {
        let ts = extract_chat_timestamp("I. Boost IV_20260325_204217_1698894137.txt", "I. Boost IV");
        assert_eq!(ts.as_deref(), Some("20260325_204217"));
    }

    #[test]
    fn parses_game_timestamp_and_character_id() {
        let ts = extract_game_timestamp("20260324_162041_2115763878.txt");
        let cid = parse_trailing_character_id(Some("20260324_162041_2115763878.txt"));
        assert_eq!(ts.as_deref(), Some("20260324_162041"));
        assert_eq!(cid, Some(2115763878));
    }

    #[test]
    fn parses_cloak_and_jump_lines() {
        assert_eq!(
            parse_jump_destination("Notify Jumping from N-RAEL to 4-07MU"),
            Some("4-07MU".to_string())
        );
        assert_eq!(parse_cloak_state("You cannot do that while cloaked"), Some(true));
        assert!(is_decloak_flash_trigger(
            "(notify) Your cloak deactivates due to proximity to an object."
        ));
    }

    #[test]
    fn splits_chat_timestamp_and_message() {
        let (ts, msg) = split_chat_timestamp_and_message("[ 2026.03.30 12:22:07 ] intel ping");
        assert_eq!(ts, "2026.03.30 12:22:07");
        assert_eq!(msg, "intel ping");
    }
}
