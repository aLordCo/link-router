use std::path::PathBuf;

use crate::core::domain::Rule;
use crate::ports::ConfigRepositoryPort;

pub struct JsonConfigRepository {
    path: PathBuf,
}

impl JsonConfigRepository {
    pub fn new() -> Self {
        Self {
            path: config_dir().join("linkrouter").join("config.json"),
        }
    }

    #[allow(dead_code)]
    pub fn new_at(path: PathBuf) -> Self {
        Self { path }
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    fn load_or_create(&self) -> Result<Vec<Rule>, Box<dyn std::error::Error + Send + Sync>> {
        match std::fs::read_to_string(&self.path) {
            Ok(contents) => Ok(serde_json::from_str::<Vec<Rule>>(&contents)?),
            Err(_) => {
                if let Some(parent) = self.path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let empty: Vec<Rule> = Vec::new();
                let json = serde_json::to_string_pretty(&empty)?;
                std::fs::write(&self.path, json)?;
                Ok(empty)
            }
        }
    }
}

impl Default for JsonConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigRepositoryPort for JsonConfigRepository {
    fn load_rules(&self) -> Result<Vec<Rule>, Box<dyn std::error::Error + Send + Sync>> {
        self.load_or_create()
    }

    fn save_rules(&self, rules: &[Rule]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(rules)?;
        std::fs::write(&self.path, json)?;
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

    #[test]
    fn roundtrips_rules_to_disk() {
        let dir = std::env::temp_dir().join(format!("linkrouter-test-{}", std::process::id()));
        let path = dir.join("config.json");
        let repo = JsonConfigRepository::new_at(path.clone());

        let rules = vec![sample_rule()];
        repo.save_rules(&rules).unwrap();
        let loaded = repo.load_rules().unwrap();
        assert_eq!(loaded, rules);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn creates_empty_file_when_missing() {
        let dir = std::env::temp_dir().join(format!("linkrouter-empty-{}", std::process::id()));
        let path = dir.join("config.json");
        let repo = JsonConfigRepository::new_at(path.clone());
        assert!(repo.load_rules().unwrap().is_empty());
        assert!(path.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
