use std::error::Error;

use url::Url;

use crate::core::domain::{BrowserProfile, Rule};

pub trait BrowserDetectorPort: Send + Sync {
    fn detect(&self) -> Result<Vec<BrowserProfile>, Box<dyn Error + Send + Sync>>;
}

pub trait UrlLauncherPort: Send + Sync {
    fn launch(
        &self,
        profile: &BrowserProfile,
        url: &Url,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub trait ConfigRepositoryPort: Send + Sync {
    fn load_rules(&self) -> Result<Vec<Rule>, Box<dyn Error + Send + Sync>>;
    #[allow(dead_code)]
    fn save_rules(&self, rules: &[Rule]) -> Result<(), Box<dyn Error + Send + Sync>>;
}
