use std::path::{Path, PathBuf};

#[cfg(any(target_os = "linux", test))]
#[derive(Debug, Clone, Default)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: String,
    pub mime_types: String,
}

#[cfg(any(target_os = "linux", test))]
impl DesktopEntry {
    pub fn is_browser(&self) -> bool {
        self.categories
            .split(';')
            .any(|category| category.trim().eq_ignore_ascii_case("WebBrowser"))
            || self.mime_types.split(';').any(|mime| {
                let mime = mime.trim();
                mime == "x-scheme-handler/http" || mime == "x-scheme-handler/https"
            })
    }
}

#[cfg(any(target_os = "linux", test))]
pub fn parse_desktop(content: &str) -> Option<DesktopEntry> {
    let mut entry: Option<DesktopEntry> = None;
    let mut in_entry = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            in_entry = line.trim_matches(|c| c == '[' || c == ']').trim() == "Desktop Entry";
            if in_entry && entry.is_none() {
                entry = Some(DesktopEntry::default());
            }
            continue;
        }

        let Some(current) = entry.as_mut() else {
            continue;
        };
        if !in_entry {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "Name" => current.name = value.to_string(),
            "Exec" => current.exec = value.to_string(),
            "Icon" => current.icon = Some(value.to_string()),
            "Categories" => current.categories = value.to_string(),
            "MimeType" => current.mime_types = value.to_string(),
            _ => {}
        }
    }

    let entry = entry?;
    if entry.exec.is_empty() {
        return None;
    }
    Some(entry)
}

#[cfg(any(target_os = "linux", test))]
pub fn desktop_exec_command(exec: &str) -> Option<String> {
    let mut tokens = exec.split_whitespace();
    let mut first = tokens.next()?;

    if first == "env" {
        for token in tokens.by_ref() {
            if !token.contains('=') {
                first = token;
                break;
            }
        }
    }

    let cleaned = first.trim_matches('"').trim_matches('\'');
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

#[cfg(any(target_os = "linux", test))]
pub fn find_in_paths(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    if name.contains('/') {
        let candidate = PathBuf::from(name);
        return candidate.is_file().then_some(candidate);
    }

    for dir in dirs {
        let candidate = dir.join(name);
        if candidate.is_file() {
            if let Ok(metadata) = std::fs::metadata(&candidate) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if metadata.permissions().mode() & 0o111 == 0 {
                        continue;
                    }
                }
                #[cfg(not(unix))]
                let _ = metadata;
                return Some(candidate);
            }
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChromiumProfile {
    pub directory: String,
    pub display_name: String,
}

fn chromium_rank(directory: &str) -> (u8, usize) {
    if directory == "Default" {
        (0, 0)
    } else if let Some(rest) = directory.strip_prefix("Profile ") {
        match rest.parse::<usize>() {
            Ok(number) => (1, number),
            Err(_) => (2, usize::MAX),
        }
    } else {
        (2, usize::MAX)
    }
}

pub fn chromium_profiles_from_local_state(local_state: &str) -> Option<Vec<ChromiumProfile>> {
    let value: serde_json::Value = serde_json::from_str(local_state).ok()?;
    let cache = value.get("profile")?.get("info_cache")?.as_object()?;

    let mut profiles: Vec<ChromiumProfile> = cache
        .iter()
        .map(|(directory, info)| ChromiumProfile {
            directory: directory.clone(),
            display_name: info
                .get("name")
                .and_then(|name| name.as_str())
                .filter(|name| !name.is_empty())
                .unwrap_or(directory)
                .to_string(),
        })
        .collect();

    profiles.sort_by_key(|profile| chromium_rank(&profile.directory));
    Some(profiles)
}

pub fn chromium_profiles(config_dir: &Path) -> Vec<ChromiumProfile> {
    if let Ok(contents) = std::fs::read(config_dir.join("Local State")) {
        if let Some(profiles) =
            chromium_profiles_from_local_state(&String::from_utf8_lossy(&contents))
        {
            if !profiles.is_empty() {
                return profiles;
            }
        }
    }

    let mut profiles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let directory = entry.file_name().to_string_lossy().into_owned();
            if directory == "Default" || directory.starts_with("Profile ") {
                profiles.push(ChromiumProfile {
                    display_name: directory.clone(),
                    directory,
                });
            }
        }
    }
    profiles.sort_by_key(|profile| chromium_rank(&profile.directory));
    profiles
}

#[derive(Debug, Clone, PartialEq)]
pub struct FirefoxProfile {
    pub name: String,
    pub dir: PathBuf,
}

pub fn parse_profiles_ini(content: &str, base: &Path) -> Vec<FirefoxProfile> {
    #[derive(Default)]
    struct Section {
        name: String,
        path: String,
        is_relative: bool,
    }

    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Section> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            if let Some(section) = current.take() {
                if !section.name.is_empty() && !section.path.is_empty() {
                    sections.push(section);
                }
            }
            current = Some(Section::default());
            continue;
        }

        let Some(section) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "Name" => section.name = value.to_string(),
            "Path" => section.path = value.to_string(),
            "IsRelative" => section.is_relative = value != "0",
            _ => {}
        }
    }
    if let Some(section) = current {
        if !section.name.is_empty() && !section.path.is_empty() {
            sections.push(section);
        }
    }

    sections
        .into_iter()
        .map(|section| FirefoxProfile {
            name: section.name,
            dir: if section.is_relative {
                base.join(&section.path)
            } else {
                PathBuf::from(&section.path)
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn parses_browser_desktop_entry() {
        let content = "\
[Desktop Entry]
Type=Application
Name=Firefox
Exec=/usr/bin/firefox %u
Icon=firefox
Categories=Network;WebBrowser;
MimeType=text/html;x-scheme-handler/http;
";
        let entry = parse_desktop(content).unwrap();
        assert!(entry.is_browser());
        assert_eq!(entry.name, "Firefox");
        assert_eq!(entry.icon.as_deref(), Some("firefox"));
    }

    #[test]
    fn rejects_non_browser_desktop_entry() {
        let content = "\
[Desktop Entry]
Type=Application
Name=LibreOffice
Exec=/usr/bin/loimpress
Categories=Office;
";
        let entry = parse_desktop(content).unwrap();
        assert!(!entry.is_browser());
    }

    #[test]
    fn extracts_exec_command_with_env_and_quotes() {
        assert_eq!(
            desktop_exec_command("/usr/bin/chromium %U").as_deref(),
            Some("/usr/bin/chromium")
        );
        assert_eq!(
            desktop_exec_command("env BAMF_DESKTOP_FILE_HINT=1 google-chrome %U").as_deref(),
            Some("google-chrome")
        );
        assert_eq!(
            desktop_exec_command("\"/opt/app/bin\"").as_deref(),
            Some("/opt/app/bin")
        );
        assert!(desktop_exec_command("").is_none());
    }

    #[test]
    fn finds_executable_in_paths() {
        let dir = fixture();
        let bin = dir.path().join("my-browser");
        std::fs::write(&bin, "").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&bin).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&bin, perms).unwrap();
        }
        assert_eq!(
            find_in_paths("my-browser", &[dir.path().to_path_buf()]).unwrap(),
            bin
        );
        assert!(find_in_paths("missing-browser", &[dir.path().to_path_buf()]).is_none());
    }

    #[test]
    fn reads_chromium_profiles_from_local_state() {
        let local_state = r#"{
  "profile": {
    "info_cache": {
      "Profile 1": { "name": "Trabajo" },
      "Default": { "name": "Default" }
    }
  }
}"#;
        let profiles = chromium_profiles_from_local_state(local_state).unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].directory, "Default");
        assert_eq!(profiles[1].directory, "Profile 1");
        assert_eq!(profiles[1].display_name, "Trabajo");
    }

    #[test]
    fn falls_back_to_directory_scan_without_local_state() {
        let dir = fixture();
        let config = dir.path().join("google-chrome");
        std::fs::create_dir_all(config.join("Default")).unwrap();
        std::fs::create_dir_all(config.join("Profile 1")).unwrap();

        let profiles = chromium_profiles(&config);
        assert_eq!(profiles[0].directory, "Default");
        assert_eq!(profiles[1].directory, "Profile 1");
    }

    #[test]
    fn returns_empty_when_no_profiles_exist() {
        let dir = fixture();
        assert!(chromium_profiles(dir.path()).is_empty());
    }

    #[test]
    fn parses_firefox_profiles_ini() {
        let content = "\
[General]
StartWithLastProfile=1

[Profile0]
Name=default-release
IsRelative=1
Path=abc123.default-release

[Profile1]
Name=Trabajo
IsRelative=0
Path=/home/alord/work-profile
";
        let base = PathBuf::from("/home/alord/.mozilla/firefox");
        let profiles = parse_profiles_ini(content, &base);
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].name, "default-release");
        assert_eq!(
            profiles[0].dir,
            PathBuf::from("/home/alord/.mozilla/firefox/abc123.default-release")
        );
        assert_eq!(profiles[1].name, "Trabajo");
        assert_eq!(profiles[1].dir, PathBuf::from("/home/alord/work-profile"));
    }
}
