use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
#[cfg(windows)]
use winreg::RegKey;

use crate::core::domain::BrowserProfile;
use crate::ports::BrowserDetectorPort;

use super::common::{chromium_profiles, parse_profiles_ini};

const REGISTRY_PATH: &str = r"SOFTWARE\Clients\StartMenuInternet";

const FALLBACK_INSTALLS: &[(&str, &str, &str)] = &[
    (
        "chrome",
        "Google Chrome",
        "Google/Chrome/Application/chrome.exe",
    ),
    (
        "brave",
        "Brave",
        "BraveSoftware/Brave-Browser/Application/brave.exe",
    ),
    (
        "edge",
        "Microsoft Edge",
        "Microsoft/Edge/Application/msedge.exe",
    ),
    ("chromium", "Chromium", "Chromium/Application/chrome.exe"),
    ("firefox", "Firefox", "Mozilla Firefox/firefox.exe"),
    ("vivaldi", "Vivaldi", "Vivaldi/Application/vivaldi.exe"),
    ("opera", "Opera", "Opera/launcher.exe"),
];

pub struct WindowsBrowserDetector;

impl WindowsBrowserDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_from(
        &self,
        local: &Path,
        roaming: &Path,
        program_files: &[PathBuf],
    ) -> Vec<BrowserProfile> {
        let mut by_exe: HashMap<PathBuf, String> = HashMap::new();

        for (display, command) in registry_browsers() {
            if let Some(executable) = parse_reg_command(&command) {
                if executable.is_file() {
                    by_exe.entry(executable).or_insert(display);
                }
            }
        }

        for root in program_files {
            for (_, display, relative) in FALLBACK_INSTALLS {
                let executable = root.join(relative);
                if executable.is_file() {
                    by_exe
                        .entry(executable)
                        .or_insert_with(|| (*display).to_string());
                }
            }
        }

        let profiles: Vec<BrowserProfile> = by_exe
            .into_iter()
            .flat_map(|(executable, name)| profiles_for_browser(local, roaming, &executable, &name))
            .collect();

        let mut unique = dedup(profiles);
        unique.sort_by(|a, b| a.id.cmp(&b.id));
        unique
    }
}

impl Default for WindowsBrowserDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserDetectorPort for WindowsBrowserDetector {
    fn detect(&self) -> Result<Vec<BrowserProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let empty = PathBuf::from("");

        let local = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| empty.clone());
        let roaming = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| empty.clone());

        let mut program_files = Vec::new();
        for key in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(value) = std::env::var_os(key) {
                program_files.push(PathBuf::from(value));
            }
        }

        Ok(self.detect_from(&local, &roaming, &program_files))
    }
}

#[cfg(windows)]
fn registry_browsers() -> Vec<(String, String)> {
    let mut browsers: Vec<(String, String)> = Vec::new();
    let mut seen = HashSet::new();

    for (hive, _) in [(HKEY_LOCAL_MACHINE, "hklm"), (HKEY_CURRENT_USER, "hkcu")] {
        let Ok(interfaces) = RegKey::predef(hive).open_subkey_with_flags(REGISTRY_PATH, KEY_READ)
        else {
            continue;
        };

        for subkey in interfaces.enum_keys().filter_map(Result::ok) {
            if !seen.insert(subkey.clone()) {
                continue;
            }
            if let Ok(key) = interfaces.open_subkey_with_flags(&subkey, KEY_READ) {
                if let Ok(command) = key.get_value::<String, _>("") {
                    browsers.push((subkey, command));
                }
            }
        }
    }

    browsers
}

fn parse_reg_command(command: &str) -> Option<PathBuf> {
    let trimmed = command.trim();
    if trimmed.starts_with('"') {
        let end = trimmed[1..].find('"')? + 1;
        let path = &trimmed[1..end];
        if path.is_empty() {
            return None;
        }
        Some(PathBuf::from(path))
    } else {
        trimmed
            .split_whitespace()
            .next()
            .map(PathBuf::from)
            .filter(|path| !path.as_os_str().is_empty())
    }
}

fn chromium_data_dir(bin: &str) -> Option<&'static str> {
    match bin {
        "chrome" => Some("Google/Chrome/User Data"),
        "chromium" => Some("Chromium/User Data"),
        "msedge" => Some("Microsoft/Edge/User Data"),
        "brave" => Some("BraveSoftware/Brave-Browser/User Data"),
        "vivaldi" => Some("Vivaldi/User Data"),
        _ => None,
    }
}

fn exe_stem(executable: &Path) -> String {
    executable
        .file_stem()
        .map(|name| name.to_string_lossy().into_owned().to_lowercase())
        .unwrap_or_default()
}

fn browser_profile(
    id: String,
    name: String,
    executable: &Path,
    profile_dir: Option<PathBuf>,
    args: Vec<String>,
) -> BrowserProfile {
    BrowserProfile {
        id,
        name,
        browser_name: executable
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default(),
        executable: executable.to_path_buf(),
        profile_dir,
        icon: None,
        args,
    }
}

fn profiles_for_browser(
    local: &Path,
    roaming: &Path,
    executable: &Path,
    name: &str,
) -> Vec<BrowserProfile> {
    let bin = exe_stem(executable);
    let default_id = format!("windows/{bin}");
    let mut profiles = Vec::new();

    if bin == "firefox" {
        let base = roaming.join("Mozilla").join("Firefox");
        if let Ok(content) = std::fs::read_to_string(base.join("profiles.ini")) {
            let detected = parse_profiles_ini(&content, &base);
            if !detected.is_empty() {
                for profile in detected {
                    profiles.push(browser_profile(
                        format!("{default_id}/{}", profile.name.replace(' ', "-")),
                        format!("{name} · {}", profile.name),
                        executable,
                        Some(profile.dir),
                        vec!["-P".to_string(), profile.name],
                    ));
                }
                return profiles;
            }
        }
    }

    if let Some(relative_dir) = chromium_data_dir(&bin) {
        let data_dir = local.join(relative_dir);
        let detected = chromium_profiles(&data_dir);

        if detected.is_empty() {
            profiles.push(browser_profile(
                default_id,
                name.to_string(),
                executable,
                None,
                Vec::new(),
            ));
        } else {
            for profile in detected {
                profiles.push(browser_profile(
                    format!("{default_id}/{}", profile.directory.replace(' ', "-")),
                    format!("{name} · {}", profile.display_name),
                    executable,
                    Some(data_dir.join(&profile.directory)),
                    vec![format!("--profile-directory={}", profile.directory)],
                ));
            }
        }
        return profiles;
    }

    profiles.push(browser_profile(
        default_id,
        name.to_string(),
        executable,
        None,
        Vec::new(),
    ));
    profiles
}

fn dedup(profiles: Vec<BrowserProfile>) -> Vec<BrowserProfile> {
    let mut seen = std::collections::HashSet::new();
    profiles
        .into_iter()
        .filter(|profile| seen.insert(profile.id.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_registry_command() {
        let command = r#""C:\Program Files\Google\Chrome\Application\chrome.exe" --flag"#;
        assert_eq!(
            parse_reg_command(command).unwrap(),
            PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe")
        );
    }

    #[test]
    fn parses_unquoted_registry_command() {
        assert_eq!(
            parse_reg_command(r"C:\Edge\msedge.exe -something").unwrap(),
            PathBuf::from(r"C:\Edge\msedge.exe")
        );
    }

    #[test]
    fn maps_known_install_suffixes() {
        let roots = [PathBuf::from(r"C:\Program Files")];
        let detector = WindowsBrowserDetector::new();
        let found = detector.detect_from(&PathBuf::from(""), &PathBuf::from(""), &roots);
        assert!(found.is_empty());
    }
}
