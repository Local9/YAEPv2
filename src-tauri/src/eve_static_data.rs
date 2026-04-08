//! Download and extract EVE Online static JSONL data next to the application executable.
//! See: https://developers.eveonline.com/docs/services/static-data/

use crate::db::DbService;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use zip::read::ZipArchive;

pub const EVE_STATIC_DATA_ZIP_URL: &str =
    "https://developers.eveonline.com/static-data/eve-online-static-data-latest-jsonl.zip";

/// Official manifest for the latest static data build (JSONL, one JSON object per line).
pub const EVE_STATIC_DATA_LATEST_JSONL_URL: &str =
    "https://developers.eveonline.com/static-data/tranquility/latest.jsonl";

const APP_SETTINGS_REMOTE_BUILD: &str = "EveSdeRemoteBuildNumber";
const APP_SETTINGS_REMOTE_RELEASE_DATE: &str = "EveSdeRemoteReleaseDate";
const APP_SETTINGS_REMOTE_KEY: &str = "EveSdeRemoteKey";
const APP_SETTINGS_CATALOG_UPDATE_PENDING: &str = "EveSdeCatalogUpdatePending";

const MAX_DOWNLOAD_BYTES: u64 = 900 * 1024 * 1024;

static DOWNLOAD_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Returns `true` if this call acquired the download lock (no other download running).
pub fn try_begin_download() -> bool {
    DOWNLOAD_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

pub fn finish_download() {
    DOWNLOAD_IN_PROGRESS.store(false, Ordering::SeqCst);
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EveStaticDataProgressPayload {
    pub phase: String,
    pub bytes_received: u64,
    pub total_bytes: Option<u64>,
    pub extract_index: Option<u64>,
    pub extract_total: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EveStaticDataDownloadDoneEvent {
    pub ok: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveSdeRemoteInfo {
    #[serde(rename = "_key")]
    pub key: String,
    pub build_number: i64,
    pub release_date: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EveStaticDataStatus {
    pub path: String,
    pub installed: bool,
    /// Unix timestamp (seconds) when the archive was installed, if known.
    pub downloaded_at_unix: Option<u64>,
    pub offer_dismissed: bool,
    /// Latest SDE build info from developers.eveonline.com (cached in SQLite after each sync).
    pub remote_sde: Option<EveSdeRemoteInfo>,
    /// True when the remote build number changed since the last stored value (user can dismiss).
    pub sde_catalog_update_pending: bool,
    /// Rows in `EveSdeTypes` (0 means PI / lookups have no cached type names until import or seed).
    pub sde_sqlite_types_count: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    downloaded_at_unix: u64,
    source_url: String,
}

fn base_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let parent = exe
        .parent()
        .ok_or_else(|| "Executable path has no parent directory".to_string())?;
    Ok(parent.join("eve-static-data"))
}

/// Directory containing extracted EVE static JSONL files (`types.jsonl`, etc.).
pub fn static_data_dir() -> Result<PathBuf, String> {
    base_dir()
}

/// Locate `types.jsonl` and `groups.jsonl` under `static_root`: either directly in `static_root`
/// or in a single immediate subdirectory (matches common ZIP layouts).
pub fn resolve_sde_jsonl_paths(static_root: &Path) -> Option<(PathBuf, PathBuf)> {
    let direct_t = static_root.join("types.jsonl");
    let direct_g = static_root.join("groups.jsonl");
    if direct_t.is_file() && direct_g.is_file() {
        return Some((direct_t, direct_g));
    }
    let entries = fs::read_dir(static_root).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let t = p.join("types.jsonl");
        let g = p.join("groups.jsonl");
        if t.is_file() && g.is_file() {
            return Some((t, g));
        }
    }
    None
}

fn manifest_path(root: &Path) -> PathBuf {
    root.join("eve-static-data-manifest.json")
}

fn local_install_status() -> Result<(String, bool, Option<u64>), String> {
    let dir = base_dir()?;
    let path_str = dir.display().to_string();
    let mpath = manifest_path(&dir);
    if !mpath.exists() {
        return Ok((path_str, false, None));
    }
    let raw = fs::read_to_string(&mpath).map_err(|e| e.to_string())?;
    let m: Manifest = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok((path_str, true, Some(m.downloaded_at_unix)))
}

fn read_cached_remote(db: &DbService) -> Result<Option<EveSdeRemoteInfo>, String> {
    let build = db
        .get_app_setting(APP_SETTINGS_REMOTE_BUILD.to_string())?
        .filter(|s| !s.trim().is_empty());
    let release_date = db
        .get_app_setting(APP_SETTINGS_REMOTE_RELEASE_DATE.to_string())?
        .filter(|s| !s.trim().is_empty());
    let key = db
        .get_app_setting(APP_SETTINGS_REMOTE_KEY.to_string())?
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "sde".to_string());
    match (build, release_date) {
        (Some(b), Some(rd)) => {
            let build_number = b.parse::<i64>().map_err(|e| e.to_string())?;
            Ok(Some(EveSdeRemoteInfo {
                key,
                build_number,
                release_date: rd,
            }))
        }
        _ => Ok(None),
    }
}

fn catalog_update_pending_from_db(db: &DbService) -> bool {
    db.get_app_setting(APP_SETTINGS_CATALOG_UPDATE_PENDING.to_string())
        .ok()
        .flatten()
        .is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

/// Fetch latest.jsonl, persist build metadata, and set `EveSdeCatalogUpdatePending` when the build changes.
pub fn sync_remote_catalog(db: &DbService) -> Result<(), String> {
    let old_build = db
        .get_app_setting(APP_SETTINGS_REMOTE_BUILD.to_string())?
        .and_then(|s| s.parse::<i64>().ok());
    let info = fetch_latest_jsonl_manifest()?;
    let transition = old_build.is_some() && old_build != Some(info.build_number);
    if transition {
        db.set_app_setting(
            APP_SETTINGS_CATALOG_UPDATE_PENDING.to_string(),
            "true".to_string(),
        )?;
    }
    db.set_app_setting(
        APP_SETTINGS_REMOTE_BUILD.to_string(),
        info.build_number.to_string(),
    )?;
    db.set_app_setting(
        APP_SETTINGS_REMOTE_RELEASE_DATE.to_string(),
        info.release_date.clone(),
    )?;
    db.set_app_setting(APP_SETTINGS_REMOTE_KEY.to_string(), info.key.clone())?;
    Ok(())
}

/// Local install status plus remote SDE manifest from DB after a best-effort network sync.
pub fn build_status(db: &DbService, offer_dismissed: bool) -> Result<EveStaticDataStatus, String> {
    if let Err(e) = sync_remote_catalog(db) {
        eprintln!("EVE SDE catalog sync failed: {e}");
    }
    let (path_str, installed, downloaded_at_unix) = local_install_status()?;
    let remote_sde = read_cached_remote(db)?;
    let sde_catalog_update_pending = catalog_update_pending_from_db(db);
    let sde_sqlite_types_count = db.eve_sde_types_count().unwrap_or(0);
    Ok(EveStaticDataStatus {
        path: path_str,
        installed,
        downloaded_at_unix,
        offer_dismissed,
        remote_sde,
        sde_catalog_update_pending,
        sde_sqlite_types_count,
    })
}

fn fetch_latest_jsonl_manifest() -> Result<EveSdeRemoteInfo, String> {
    let mut response = ureq::get(EVE_STATIC_DATA_LATEST_JSONL_URL)
        .header("User-Agent", "yaep-rust")
        .call()
        .map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "SDE manifest request failed with HTTP {}",
            status.as_u16()
        ));
    }
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let info: EveSdeRemoteInfo = serde_json::from_str(line).map_err(|e| e.to_string())?;
        if info.key == "sde" {
            return Ok(info);
        }
    }
    Err("SDE manifest did not contain a valid \"sde\" entry".to_string())
}

pub fn dismiss_catalog_update_notice(db: &DbService) -> Result<(), String> {
    db.set_app_setting(
        APP_SETTINGS_CATALOG_UPDATE_PENDING.to_string(),
        "false".to_string(),
    )
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn emit_progress(app: &AppHandle, payload: EveStaticDataProgressPayload) {
    let _ = app.emit("eve-static-data-progress", &payload);
}

fn extract_zip_file_with_progress(
    app: &AppHandle,
    zip_path: &Path,
    out_dir: &Path,
) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let total = archive.len() as u64;
    let mut last_emit = Instant::now();
    for i in 0..archive.len() {
        let should_emit = last_emit.elapsed() >= Duration::from_millis(200)
            || (i as u64) % 40 == 0
            || i + 1 == archive.len();
        if should_emit {
            emit_progress(
                app,
                EveStaticDataProgressPayload {
                    phase: "extracting".to_string(),
                    bytes_received: 0,
                    total_bytes: None,
                    extract_index: Some(i as u64),
                    extract_total: Some(total),
                },
            );
            last_emit = Instant::now();
        }
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(rel) = file.enclosed_name().map(|p| p.to_path_buf()) else {
            continue;
        };
        let outpath = out_dir.join(&rel);
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Download the official ZIP, replace `eve-static-data` under the executable directory, and write a manifest.
/// Emits [`EveStaticDataProgressPayload`] on `eve-static-data-progress` during download and extraction.
pub fn download_and_install_with_emit(app: &AppHandle) -> Result<(), String> {
    let dir = base_dir()?;
    let tmp_zip = std::env::temp_dir().join("yaep-eve-static-data-latest.zip");

    {
        let mut response = ureq::get(EVE_STATIC_DATA_ZIP_URL)
            .header("User-Agent", "yaep-rust")
            .call()
            .map_err(|e| e.to_string())?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!(
                "Download failed with HTTP {}",
                status.as_u16()
            ));
        }

        let total_bytes: Option<u64> = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let mut reader = response
            .body_mut()
            .with_config()
            .limit(MAX_DOWNLOAD_BYTES)
            .reader();
        let mut file = File::create(&tmp_zip).map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; 256 * 1024];
        let mut downloaded: u64 = 0;
        let mut last_emit = Instant::now();

        emit_progress(
            app,
            EveStaticDataProgressPayload {
                phase: "downloading".to_string(),
                bytes_received: 0,
                total_bytes,
                extract_index: None,
                extract_total: None,
            },
        );

        loop {
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            downloaded += n as u64;
            if last_emit.elapsed() >= Duration::from_millis(200) {
                emit_progress(
                    app,
                    EveStaticDataProgressPayload {
                        phase: "downloading".to_string(),
                        bytes_received: downloaded,
                        total_bytes,
                        extract_index: None,
                        extract_total: None,
                    },
                );
                last_emit = Instant::now();
            }
        }

        emit_progress(
            app,
            EveStaticDataProgressPayload {
                phase: "downloading".to_string(),
                bytes_received: downloaded,
                total_bytes,
                extract_index: None,
                extract_total: None,
            },
        );
    }

    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    extract_zip_file_with_progress(app, &tmp_zip, &dir)?;

    let _ = fs::remove_file(&tmp_zip);

    let manifest = Manifest {
        downloaded_at_unix: now_unix_secs(),
        source_url: EVE_STATIC_DATA_ZIP_URL.to_string(),
    };
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    let mut mf = File::create(manifest_path(&dir)).map_err(|e| e.to_string())?;
    mf.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}
