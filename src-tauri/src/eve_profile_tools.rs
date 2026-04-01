use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sysinfo::{ProcessesToUpdate, System};
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

#[derive(Default)]
pub struct EveProfileToolsService;

impl EveProfileToolsService {
    /// Public ESI: character name by ID (no auth).
    pub fn fetch_character_name(&self, character_id: u64) -> Result<String, String> {
        let url = format!("https://esi.evetech.net/latest/characters/{character_id}/");
        let v: Value = ureq::get(&url)
            .call()
            .map_err(|e| e.to_string())?
            .into_json()
            .map_err(|e| e.to_string())?;
        v.get("name")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "ESI response missing name".to_string())
    }

    pub fn list_profiles(&self) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        let base = self.eve_local_base_dir()?;
        if !base.exists() {
            return Ok(out);
        }

        for server_dir in fs::read_dir(base).map_err(|e| e.to_string())? {
            let server_dir = server_dir.map_err(|e| e.to_string())?;
            if !server_dir.path().is_dir() {
                continue;
            }
            for entry in fs::read_dir(server_dir.path()).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry.file_name().to_string_lossy().to_string();
                if !entry.path().is_dir() {
                    continue;
                }
                if let Some(profile_name) = name.strip_prefix("settings_") {
                    out.push(profile_name.to_string());
                } else if name.eq_ignore_ascii_case("Default") {
                    out.push("Default".to_string());
                }
            }
        }

        out.sort();
        out.dedup();
        Ok(out)
    }

    pub fn copy_profile(&self, source_profile: String, new_profile: String) -> Result<(), String> {
        self.ensure_eve_not_running()?;
        let source = self.find_profile_dir(&source_profile)?;
        let source_parent = source
            .parent()
            .ok_or_else(|| "Invalid source profile path".to_string())?;
        let destination = source_parent.join(format!("settings_{}", new_profile.trim()));
        if destination.exists() {
            return Err("Destination profile already exists".to_string());
        }
        copy_dir_recursive(&source, &destination)
    }

    pub fn copy_character_files(
        &self,
        source_profile: String,
        target_profile: String,
    ) -> Result<(), String> {
        self.ensure_eve_not_running()?;
        let source = self.find_profile_dir(&source_profile)?;
        let target = self.find_profile_dir(&target_profile)?;

        for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_copy_file =
                file_name.starts_with("core_char_") || file_name.starts_with("core_user_");
            if !is_copy_file || !entry.path().is_file() {
                continue;
            }
            let destination = target.join(file_name);
            fs::copy(entry.path(), destination).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn backup_all_profiles(&self, output_path: String) -> Result<(), String> {
        self.ensure_eve_not_running()?;
        let base = self.eve_local_base_dir()?;
        if !base.exists() {
            return Err("EVE local directory not found".to_string());
        }

        let mut profile_dirs: Vec<PathBuf> = Vec::new();
        for server_dir in fs::read_dir(&base).map_err(|e| e.to_string())? {
            let server_dir = server_dir.map_err(|e| e.to_string())?;
            if !server_dir.path().is_dir() {
                continue;
            }
            for entry in fs::read_dir(server_dir.path()).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name.eq_ignore_ascii_case("Default") || name.starts_with("settings_") {
                    profile_dirs.push(entry.path());
                }
            }
        }

        if profile_dirs.is_empty() {
            return Err("No EVE profiles found to back up".to_string());
        }

        let file = fs::File::create(&output_path).map_err(|e| e.to_string())?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        for profile_dir in &profile_dirs {
            self.add_directory_to_zip(profile_dir, &base, &mut zip, options)?;
        }

        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn find_profile_dir(&self, profile_name: &str) -> Result<PathBuf, String> {
        let normalized = profile_name.trim();
        let base = self.eve_local_base_dir()?;
        if !base.exists() {
            return Err("EVE local directory not found".to_string());
        }

        for server_dir in fs::read_dir(base).map_err(|e| e.to_string())? {
            let server_dir = server_dir.map_err(|e| e.to_string())?;
            if !server_dir.path().is_dir() {
                continue;
            }
            for entry in fs::read_dir(server_dir.path()).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry.file_name().to_string_lossy().to_string();
                if !entry.path().is_dir() {
                    continue;
                }
                if name.eq_ignore_ascii_case("Default")
                    && normalized.eq_ignore_ascii_case("Default")
                {
                    return Ok(entry.path());
                }
                if name == format!("settings_{normalized}") {
                    return Ok(entry.path());
                }
            }
        }

        Err(format!("Profile not found: {normalized}"))
    }

    fn eve_local_base_dir(&self) -> Result<PathBuf, String> {
        let local_appdata = std::env::var("LOCALAPPDATA")
            .map_err(|_| "LOCALAPPDATA environment variable not found".to_string())?;
        Ok(PathBuf::from(local_appdata).join("CCP").join("EVE"))
    }

    fn ensure_eve_not_running(&self) -> Result<(), String> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);
        let running = system.processes().values().any(|p| {
            p.name()
                .to_string_lossy()
                .eq_ignore_ascii_case("exefile.exe")
                || p.name().to_string_lossy().eq_ignore_ascii_case("exefile")
        });
        if running {
            return Err("Cannot run profile tools while exefile is running".to_string());
        }
        Ok(())
    }

    fn add_directory_to_zip(
        &self,
        source_dir: &Path,
        base_dir: &Path,
        zip: &mut ZipWriter<fs::File>,
        options: FileOptions,
    ) -> Result<(), String> {
        for entry in fs::read_dir(source_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let relative = path
                .strip_prefix(base_dir)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");

            if path.is_dir() {
                zip.add_directory(format!("{relative}/"), options)
                    .map_err(|e| e.to_string())?;
                self.add_directory_to_zip(&path, base_dir, zip, options)?;
            } else {
                zip.start_file(relative, options).map_err(|e| e.to_string())?;
                let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, zip).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
