use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::domain::BrowserProfile;
use crate::ports::BrowserDetectorPort;

use super::common::{
    chromium_profiles, desktop_exec_command, find_in_paths, parse_desktop, parse_profiles_ini,
    FirefoxProfile,
};

const KNOWN_BINARIES: &[(&str, &[&str])] = &[
    (
        "Chromium",
        &["chromium", "chromium-browser", "chromium-stable"],
    ),
    ("Google Chrome", &["google-chrome", "google-chrome-stable"]),
    ("Brave", &["brave-browser", "brave"]),
    ("Firefox", &["firefox"]),
    (
        "Microsoft Edge",
        &["microsoft-edge", "microsoft-edge-stable"],
    ),
    ("Vivaldi", &["vivaldi", "vivaldi-stable"]),
    ("Opera", &["opera"]),
    ("Zen Browser", &["zen-browser"]),
    ("Arc", &["arc"]),
];

const APPLICATIONS_DIRS: &[&str] = &[
    "/usr/share/applications",
    "/usr/local/share/applications",
    "/var/lib/snapd/desktop/applications",
    "/var/lib/flatpak/exports/share/applications",
    "~/.local/share/applications",
    "~/.local/share/flatpak/exports/share/applications",
];

pub struct LinuxBrowserDetector;

impl LinuxBrowserDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_from(
        &self,
        home: &Path,
        applications: &[PathBuf],
        path_dirs: &[PathBuf],
    ) -> Vec<BrowserProfile> {
        let mut by_exec: HashMap<PathBuf, (String, Option<String>)> = HashMap::new();

        for dir in applications {
            for path in list_desktop_files(dir) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(entry) = parse_desktop(&content) {
                        if entry.is_browser() {
                            if let Some(command) = desktop_exec_command(&entry.exec) {
                                if let Some(executable) = resolve_executable(&command, path_dirs) {
                                    by_exec
                                        .entry(executable)
                                        .or_insert_with(|| (entry.name, entry.icon));
                                }
                            }
                        }
                    }
                }
            }
        }

        for &(name, binaries) in KNOWN_BINARIES {
            for binary in binaries {
                if let Some(path) = find_in_paths(binary, path_dirs) {
                    by_exec
                        .entry(path)
                        .or_insert_with(|| ((*name).to_string(), None));
                    break;
                }
            }
        }

        let mut profiles: Vec<BrowserProfile> = by_exec
            .into_iter()
            .flat_map(|(executable, (name, icon))| {
                profiles_for_browser(home, &executable, &name, icon.as_deref())
            })
            .collect();

        if let Some(browser_env) =
            std::env::var_os("BROWSER").and_then(|value| value.to_str().map(str::to_string))
        {
            for entry in browser_env.split(':').take(1) {
                let executable = PathBuf::from(entry);
                if executable.is_file() {
                    profiles.push(browser_profile(
                        "env/browser",
                        "BROWSER",
                        &executable,
                        None,
                        None,
                        Vec::new(),
                    ));
                }
            }
        }

        let mut unique = dedup(profiles);
        unique.sort_by(|a, b| a.id.cmp(&b.id));
        unique
    }
}

impl Default for LinuxBrowserDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserDetectorPort for LinuxBrowserDetector {
    fn detect(&self) -> Result<Vec<BrowserProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();

        let applications: Vec<PathBuf> = APPLICATIONS_DIRS
            .iter()
            .map(|raw| match raw.strip_prefix("~/") {
                Some(rest) => home.join(rest),
                None => PathBuf::from(raw),
            })
            .collect();

        let path_dirs: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).collect())
            .unwrap_or_default();

        Ok(self.detect_from(&home, &applications, &path_dirs))
    }
}

fn list_desktop_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .map(|ext| ext == "desktop")
                    .unwrap_or(false)
            {
                files.push(path);
            }
        }
    }
    files
}

fn resolve_executable(command: &str, path_dirs: &[PathBuf]) -> Option<PathBuf> {
    if command.contains('/') {
        let candidate = PathBuf::from(command);
        return candidate.is_file().then_some(candidate);
    }
    find_in_paths(command, path_dirs)
}

fn chromium_config_dir(bin: &str) -> Option<&'static str> {
    match bin {
        "google-chrome" | "google-chrome-stable" | "chrome" => Some(".config/google-chrome"),
        "chromium" | "chromium-browser" | "chromium-stable" => Some(".config/chromium"),
        "brave-browser" | "brave" => Some(".config/BraveSoftware/Brave-Browser"),
        "vivaldi" | "vivaldi-stable" => Some(".config/vivaldi"),
        "microsoft-edge" | "microsoft-edge-stable" | "msedge" => Some(".config/microsoft-edge"),
        "opera" => Some(".config/opera"),
        "arc" => Some(".config/Arc"),
        _ => None,
    }
}

fn firefox_homes(home: &Path, bin: &str) -> Vec<PathBuf> {
    match bin {
        "firefox" => vec![home.join(".mozilla").join("firefox")],
        "zen-browser" | "zen" => vec![home.join(".zen"), home.join(".mozilla").join("zen")],
        _ => Vec::new(),
    }
}

fn bin_name(executable: &Path) -> String {
    executable
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn slug(value: &str) -> String {
    value.replace(' ', "-")
}

fn browser_profile(
    id: &str,
    name: &str,
    executable: &Path,
    icon: Option<&str>,
    profile_dir: Option<PathBuf>,
    args: Vec<String>,
) -> BrowserProfile {
    BrowserProfile {
        id: id.to_string(),
        name: name.to_string(),
        browser_name: bin_name(executable),
        executable: executable.to_path_buf(),
        profile_dir,
        icon: icon.map(str::to_string),
        args,
    }
}

fn profiles_for_browser(
    home: &Path,
    executable: &Path,
    name: &str,
    icon: Option<&str>,
) -> Vec<BrowserProfile> {
    let bin = bin_name(executable);
    let mut profiles = Vec::new();

    if let Some(relative_dir) = chromium_config_dir(&bin) {
        let config_dir = home.join(relative_dir);
        let detected = chromium_profiles(&config_dir);

        if detected.is_empty() {
            profiles.push(browser_profile(
                &format!("linux/{bin}"),
                name,
                executable,
                icon,
                None,
                Vec::new(),
            ));
        } else {
            for profile in detected {
                profiles.push(browser_profile(
                    &format!("linux/{}/{}", bin, slug(&profile.directory)),
                    &format!("{name} · {}", profile.display_name),
                    executable,
                    icon,
                    Some(config_dir.join(&profile.directory)),
                    vec![format!("--profile-directory={}", profile.directory)],
                ));
            }
        }
        return profiles;
    }

    for firefox_home in firefox_homes(home, &bin) {
        if let Ok(content) = std::fs::read_to_string(firefox_home.join("profiles.ini")) {
            let base = firefox_home;
            let detected: Vec<FirefoxProfile> = parse_profiles_ini(&content, &base);

            if detected.is_empty() {
                break;
            }

            for profile in detected {
                profiles.push(browser_profile(
                    &format!("linux/{}/{}", bin, slug(&profile.name)),
                    &format!("{name} · {}", profile.name),
                    executable,
                    icon,
                    Some(profile.dir),
                    vec!["-P".to_string(), profile.name],
                ));
            }
            return profiles;
        }
    }

    profiles.push(browser_profile(
        &format!("linux/{bin}"),
        name,
        executable,
        icon,
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
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn touch_executable(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, "").unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).unwrap();
        path
    }

    fn write_chrome_local_state(home: &Path) {
        let config = home.join(".config").join("google-chrome");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("Local State"),
            r#"{"profile":{"info_cache":{"Default":{"name":"Default"},"Profile 1":{"name":"Trabajo"}}}}"#,
        )
        .unwrap();
    }

    fn write_firefox_profiles(home: &Path) {
        let dir = home.join(".mozilla").join("firefox");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("profiles.ini"),
            "\
[General]
StartWithLastProfile=1

[Profile0]
Name=default-release
IsRelative=1
Path=abc123.default-release

[Profile1]
Name=Trabajo
IsRelative=0
Path=/tmp/nonexistent-work
",
        )
        .unwrap();
    }

    #[test]
    fn expands_chromium_profiles() {
        let home = tempfile::tempdir().unwrap();
        write_chrome_local_state(home.path());
        let bins = tempfile::tempdir().unwrap();
        touch_executable(bins.path(), "google-chrome");

        let detector = LinuxBrowserDetector::new();
        let found = detector.detect_from(home.path(), &[], &[bins.path().to_path_buf()]);

        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|p| {
            p.id == "linux/google-chrome/Default" && p.args == vec!["--profile-directory=Default"]
        }));
        assert!(found.iter().any(|p| {
            p.id == "linux/google-chrome/Profile-1"
                && p.name == "Google Chrome · Trabajo"
                && p.args == vec!["--profile-directory=Profile 1"]
                && p.profile_dir
                    == Some(home.path().join(".config/google-chrome").join("Profile 1"))
        }));
    }

    #[test]
    fn expands_firefox_profiles() {
        let home = tempfile::tempdir().unwrap();
        write_firefox_profiles(home.path());
        let bins = tempfile::tempdir().unwrap();
        touch_executable(bins.path(), "firefox");

        let detector = LinuxBrowserDetector::new();
        let found = detector.detect_from(home.path(), &[], &[bins.path().to_path_buf()]);

        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|p| {
            p.id == "linux/firefox/default-release"
                && p.args == vec!["-P", "default-release"]
                && p.profile_dir
                    == Some(
                        home.path()
                            .join(".mozilla/firefox")
                            .join("abc123.default-release"),
                    )
        }));
        assert!(found
            .iter()
            .any(|p| { p.id == "linux/firefox/Trabajo" && p.args == vec!["-P", "Trabajo"] }));
    }

    #[test]
    fn discovers_browser_from_desktop_entry() {
        let home = tempfile::tempdir().unwrap();
        write_chrome_local_state(home.path());

        let apps = tempfile::tempdir().unwrap();
        std::fs::write(
            apps.path().join("fake-browser.desktop"),
            "\
[Desktop Entry]
Type=Application
Name=Fake Browser
Exec=google-chrome %U
Icon=fake-icon
Categories=Network;WebBrowser;
MimeType=text/html;x-scheme-handler/http;
",
        )
        .unwrap();

        let bins = tempfile::tempdir().unwrap();
        touch_executable(bins.path(), "google-chrome");

        let detector = LinuxBrowserDetector::new();
        let found = detector.detect_from(
            home.path(),
            &[apps.path().to_path_buf()],
            &[bins.path().to_path_buf()],
        );

        assert!(found.iter().any(|p| {
            p.name == "Fake Browser · Trabajo" && p.icon.as_deref() == Some("fake-icon")
        }));
    }

    #[test]
    fn emits_default_entry_when_no_profiles_exist() {
        let home = tempfile::tempdir().unwrap();
        let bins = tempfile::tempdir().unwrap();
        touch_executable(bins.path(), "chromium");

        let detector = LinuxBrowserDetector::new();
        let found = detector.detect_from(home.path(), &[], &[bins.path().to_path_buf()]);

        assert_eq!(
            found,
            vec![browser_profile(
                "linux/chromium",
                "Chromium",
                &bins.path().join("chromium"),
                None,
                None,
                Vec::new()
            )]
        );
    }
}
