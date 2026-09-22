use std::error::Error;

use url::Url;

use crate::core::domain::{AppConfig, BrowserProfile};

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
    fn load_config(&self) -> Result<AppConfig, Box<dyn Error + Send + Sync>>;
    fn save_config(&self, config: &AppConfig) -> Result<(), Box<dyn Error + Send + Sync>>;
}
