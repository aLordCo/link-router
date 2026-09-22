use std::path::PathBuf;

use crate::core::domain::{AppConfig, CONFIG_VERSION, Rule};
use crate::ports::ConfigRepositoryPort;

pub struct JsonConfigRepository {
    path: PathBuf,
    legacy_path: PathBuf,
}

impl JsonConfigRepository {
    pub fn new() -> Self {
        let base = config_dir();
        Self {
            path: base.join("link-router").join("config.json"),
            legacy_path: base.join("linkrouter").join("config.json"),
        }
    }

    #[allow(dead_code)]
    pub fn new_at(path: PathBuf) -> Self {
        Self {
            path,
            legacy_path: PathBuf::new(),
        }
    }

    #[allow(dead_code)]
    pub fn new_at_with_legacy(path: PathBuf, legacy_path: PathBuf) -> Self {
        Self { path, legacy_path }
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    fn parse(contents: &str) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(config) = serde_json::from_str::<AppConfig>(contents) {
            return Ok(config);
        }
        let rules = serde_json::from_str::<Vec<Rule>>(contents)?;
        Ok(AppConfig {
            version: CONFIG_VERSION,
            settings: Default::default(),
            rules,
        })
    }

    fn migrate_legacy(&self) -> Result<Option<AppConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let contents = match std::fs::read_to_string(&self.legacy_path) {
            Ok(contents) => contents,
            Err(_) => return Ok(None),
        };
        let rules = match serde_json::from_str::<Vec<Rule>>(&contents) {
            Ok(rules) => rules,
            Err(e) => {
                log::warn!("legacy config is invalid, ignoring it: {e}");
                return Ok(None);
            }
        };
        let config = AppConfig {
            version: CONFIG_VERSION,
            settings: Default::default(),
            rules,
        };
        self.save_config(&config)?;
        let _ = std::fs::remove_file(&self.legacy_path);
        log::info!("migrated legacy config to {}", self.path.display());
        Ok(Some(config))
    }
}

impl Default for JsonConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigRepositoryPort for JsonConfigRepository {
    fn load_config(&self) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(contents) = std::fs::read_to_string(&self.path) {
            return Self::parse(&contents);
        }
        if let Some(config) = self.migrate_legacy()? {
            return Ok(config);
        }
        Ok(AppConfig::default())
    }

    fn save_config(&self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let parent = self
            .path
            .parent()
            .ok_or("config path has no parent directory")?;
        std::fs::create_dir_all(parent)?;

        let tmp = parent.join(format!(".config.{}.tmp", std::process::id()));
        let json = serde_json::to_string_pretty(config)?;
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

fn config_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg);
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".config");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata);
        }
    }

    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::AppSettings;

    fn sample_rule() -> Rule {
        Rule {
            id: "r1".into(),
            priority: 2,
            enabled: true,
            pattern: crate::core::domain::RulePattern::Domain {
                pattern: "*.corp.example".into(),
            },
            target_profile_id: "chrome-work".into(),
        }
    }

    fn sample_config() -> AppConfig {
        AppConfig {
            version: CONFIG_VERSION,
            settings: AppSettings {
                locale: "es".into(),
                theme: "dark".into(),
            },
            rules: vec![sample_rule()],
        }
    }

    #[test]
    fn roundtrips_config_to_disk() {
        let dir = std::env::temp_dir().join(format!("linkrouter-test-{}", std::process::id()));
        let path = dir.join("config.json");
        let repo = JsonConfigRepository::new_at(path.clone());

        let config = sample_config();
        repo.save_config(&config).unwrap();
        let loaded = repo.load_config().unwrap();
        assert_eq!(loaded, config);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_returns_default_without_creating() {
        let dir = std::env::temp_dir().join(format!("linkrouter-missing-{}", std::process::id()));
        let path = dir.join("config.json");
        let repo = JsonConfigRepository::new_at(path.clone());
        assert_eq!(repo.load_config().unwrap(), AppConfig::default());
        assert!(!path.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrates_legacy_array_config() {
        let dir =
            std::env::temp_dir().join(format!("linkrouter-migrate-{}", std::process::id()));
        let path = dir.join("config.json");
        let legacy = dir.join("legacy.json");
        std::fs::create_dir_all(&dir).unwrap();

        let rules = vec![sample_rule()];
        std::fs::write(&legacy, serde_json::to_string(&rules).unwrap()).unwrap();

        let repo = JsonConfigRepository::new_at_with_legacy(path.clone(), legacy.clone());
        let config = repo.load_config().unwrap();
        assert_eq!(config.rules, rules);
        assert_eq!(config.settings, AppSettings::default());
        assert!(path.is_file());
        assert!(!legacy.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_json_returns_error() {
        let dir = std::env::temp_dir().join(format!("linkrouter-invalid-{}", std::process::id()));
        let path = dir.join("config.json");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "{not valid json").unwrap();
        let repo = JsonConfigRepository::new_at(path.clone());
        assert!(repo.load_config().is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}