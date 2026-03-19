use std::fs;
use std::path::{Path, PathBuf};

use sysinfo::{ProcessesToUpdate, System};

#[derive(Default)]
pub struct EveProfileToolsService;

impl EveProfileToolsService {
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
                if name.eq_ignore_ascii_case("Default") && normalized.eq_ignore_ascii_case("Default")
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
