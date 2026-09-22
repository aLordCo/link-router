use std::path::{Path, PathBuf};

use crate::core::domain::BrowserProfile;
use crate::ports::BrowserDetectorPort;

use super::common::{chromium_profiles, parse_profiles_ini};

const BUNDLES: &[(&str, &str, &str)] = &[
    ("safari", "Safari", "Safari.app"),
    ("chrome", "Google Chrome", "Google Chrome.app"),
    ("brave", "Brave", "Brave Browser.app"),
    ("firefox", "Firefox", "Firefox.app"),
    ("edge", "Microsoft Edge", "Microsoft Edge.app"),
    ("vivaldi", "Vivaldi", "Vivaldi.app"),
    ("opera", "Opera", "Opera.app"),
    ("arc", "Arc", "Arc.app"),
    ("orion", "Orion", "Orion.app"),
    ("zen", "Zen Browser", "Zen Browser.app"),
];

const APPLICATION_SUPPORT_DIRS: &[(&str, &str)] = &[
    ("Google Chrome", "Google/Chrome"),
    ("Brave", "BraveSoftware/Brave-Browser"),
    ("Microsoft Edge", "Microsoft Edge"),
    ("Vivaldi", "Vivaldi"),
    ("Chromium", "Chromium"),
    ("Arc", "Arc/User Data"),
];

pub struct MacOsBrowserDetector;

impl MacOsBrowserDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_from(&self, home: &Path, applications: &[PathBuf]) -> Vec<BrowserProfile> {
        let application_support = home.join("Library").join("Application Support");

        let mut profiles = Vec::new();

        for (_, bundle_name, _) in BUNDLES {
            for dir in applications {
                let app_dir = dir.join(bundle_name);
                if !app_dir.is_dir() {
                    continue;
                }
                let executable = executable_of(&app_dir, bundle_name);
                let Some(executable) = executable else {
                    continue;
                };
                let icon = first_icns(&app_dir).map(|path| path.to_string_lossy().into_owned());

                profiles.extend(profiles_for_browser(
                    &application_support,
                    bundle_name,
                    &executable,
                    icon,
                ));
                break;
            }
        }

        let mut unique = dedup(profiles);
        unique.sort_by(|a, b| a.id.cmp(&b.id));
        unique
    }
}

impl Default for MacOsBrowserDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserDetectorPort for MacOsBrowserDetector {
    fn detect(&self) -> Result<Vec<BrowserProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();

        let mut applications = vec![PathBuf::from("/Applications")];
        applications.push(home.join("Applications"));

        Ok(self.detect_from(&home, &applications))
    }
}

fn executable_of(app_dir: &Path, bundle_name: &str) -> Option<PathBuf> {
    let binary = match bundle_name {
        "Safari.app" | "Safari" => "Safari",
        "Google Chrome.app" | "Google Chrome" => "Google Chrome",
        "Brave Browser.app" | "Brave" => "Brave Browser",
        "Firefox.app" | "Firefox" => "firefox",
        "Microsoft Edge.app" | "Microsoft Edge" => "Microsoft Edge",
        "Vivaldi.app" | "Vivaldi" => "Vivaldi",
        "Opera.app" | "Opera" => "Opera",
        "Arc.app" | "Arc" => "Arc",
        "Orion.app" | "Orion" => "Orion",
        "Zen Browser.app" | "Zen Browser" => "zen",
        other => other,
    };
    let candidate = app_dir.join("Contents").join("MacOS").join(binary);
    candidate.is_file().then_some(candidate)
}

fn first_icns(app_dir: &Path) -> Option<PathBuf> {
    let resources = app_dir.join("Contents").join("Resources");
    if let Ok(entries) = std::fs::read_dir(&resources) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|ext| ext == "icns").unwrap_or(false) {
                return Some(path);
            }
        }
    }
    None
}

fn chromium_app_data_dir(display_name: &str) -> Option<&'static str> {
    APPLICATION_SUPPORT_DIRS
        .iter()
        .find(|(name, _)| *name == display_name)
        .map(|(_, dir)| *dir)
}

fn browser_profile(
    id: String,
    name: String,
    executable: &Path,
    icon: Option<String>,
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
        icon,
        args,
    }
}

fn profiles_for_browser(
    application_support: &Path,
    bundle_name: &str,
    executable: &Path,
    icon: Option<String>,
) -> Vec<BrowserProfile> {
    let mut profiles = Vec::new();
    let id = format!(
        "macos/{}",
        bundle_name.to_ascii_lowercase().replace(' ', "-")
    );

    if let Some(relative_dir) = chromium_app_data_dir(bundle_name) {
        let data_dir = application_support.join(relative_dir);
        let detected = chromium_profiles(&data_dir);

        if detected.is_empty() {
            profiles.push(browser_profile(
                id,
                bundle_name.to_string(),
                executable,
                icon,
                None,
                Vec::new(),
            ));
        } else {
            for profile in detected {
                profiles.push(browser_profile(
                    format!("{id}/{}", profile.directory.replace(' ', "-")),
                    format!("{bundle_name} · {}", profile.display_name),
                    executable,
                    icon.clone(),
                    Some(data_dir.join(&profile.directory)),
                    vec![format!("--profile-directory={}", profile.directory)],
                ));
            }
        }
        return profiles;
    }

    if bundle_name == "Firefox" {
        let base = application_support.join("Firefox");
        if let Ok(content) = std::fs::read_to_string(base.join("profiles.ini")) {
            let detected = parse_profiles_ini(&content, &base);
            if !detected.is_empty() {
                for profile in detected {
                    profiles.push(browser_profile(
                        format!("{id}/{}", slug(&profile.name)),
                        format!("{bundle_name} · {}", profile.name),
                        executable,
                        icon.clone(),
                        Some(profile.dir),
                        vec!["-P".to_string(), profile.name],
                    ));
                }
                return profiles;
            }
        }
    }

    profiles.push(browser_profile(
        id,
        bundle_name.to_string(),
        executable,
        icon,
        None,
        Vec::new(),
    ));
    profiles
}

fn slug(value: &str) -> String {
    value.replace(' ', "-")
}

fn dedup(profiles: Vec<BrowserProfile>) -> Vec<BrowserProfile> {
    let mut seen = std::collections::HashSet::new();
    profiles
        .into_iter()
        .filter(|profile| seen.insert(profile.id.clone()))
        .collect()
}
