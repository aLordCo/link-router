pub mod associations;
pub mod commands;
pub mod detectors;
pub mod dispatcher;
pub mod launchers;
pub mod persistence;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::tray::TrayIcon;
use tauri::{AppHandle, Wry};

use crate::core::domain::{AppConfig, AppSettings, CONFIG_VERSION, Rule};
use crate::core::error::RouteError;
use crate::core::route_service::RouteEvaluator;
use crate::ports::{BrowserDetectorPort, ConfigRepositoryPort, UrlLauncherPort};

pub struct AppState {
    pub detector: Box<dyn BrowserDetectorPort>,
    pub launcher: Box<dyn UrlLauncherPort>,
    pub repository: Box<dyn ConfigRepositoryPort>,
    pub settings: Mutex<AppSettings>,
    pub rules: Mutex<Vec<Rule>>,
    pub evaluator: Mutex<RouteEvaluator>,
    pub config_io: Mutex<()>,
    pub tray: Mutex<Option<TrayIcon<Wry>>>,
    pub last_url: Mutex<Option<(String, Instant)>>,
}

impl AppState {
    pub fn should_handle(&self, url: &str) -> bool {
        let mut last = match self.last_url.lock() {
            Ok(lock) => lock,
            Err(_) => return false,
        };
        let now = Instant::now();
        if let Some((previous, at)) = last.as_ref() {
            if previous == url && now.duration_since(*at) < Duration::from_millis(500) {
                return false;
            }
        }
        *last = Some((url.to_string(), now));
        true
    }
}

fn poisoned() -> RouteError {
    RouteError::RuleEngine("internal state lock poisoned".into())
}

fn initial_config(repository: &dyn ConfigRepositoryPort) -> AppConfig {
    match repository.load_config() {
        Ok(config) => config,
        Err(e) => {
            log::warn!("invalid config file, using defaults: {e}");
            AppConfig::default()
        }
    }
}

fn initial_evaluator(rules: Vec<Rule>) -> RouteEvaluator {
    match RouteEvaluator::new(rules) {
        Ok(evaluator) => evaluator,
        Err(e) => {
            log::error!("invalid rule set; starting with empty rules: {e}");
            RouteEvaluator::new(Vec::new()).expect("empty rule set always compiles")
        }
    }
}

pub fn build_app_state(_handle: &AppHandle) -> AppState {
    let repository = persistence::JsonConfigRepository::new();
    let config = initial_config(&repository);
    let rules = config.rules;
    let evaluator = initial_evaluator(rules.clone());

    AppState {
        detector: Box::new(detectors::system_detector()),
        launcher: Box::new(launchers::ProcessLauncher::new()),
        repository: Box::new(repository),
        settings: Mutex::new(config.settings),
        rules: Mutex::new(rules),
        evaluator: Mutex::new(evaluator),
        config_io: Mutex::new(()),
        tray: Mutex::new(None),
        last_url: Mutex::new(None),
    }
}

pub(crate) fn effective_locale(settings: &AppSettings) -> &'static str {
    match settings.locale.as_str() {
        "es" => "es",
        "en" => "en",
        _ => {
            let tag = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
            if tag.starts_with("es") {
                "es"
            } else {
                "en"
            }
        }
    }
}

#[allow(dead_code)]
pub fn reload_rules(state: &AppState) -> Result<(), RouteError> {
    let config = state
        .repository
        .load_config()
        .map_err(|e| RouteError::RuleEngine(e.to_string()))?;
    *state.rules.lock().map_err(|_| poisoned())? = config.rules.clone();
    let evaluator = initial_evaluator(config.rules);
    *state.evaluator.lock().map_err(|_| poisoned())? = evaluator;
    Ok(())
}

pub fn commit_rules(state: &AppState) -> Result<(), RouteError> {
    let rules = state.rules.lock().map_err(|_| poisoned())?.clone();
    let evaluator = RouteEvaluator::new(rules.clone())?;
    persist_config(state)?;
    *state.evaluator.lock().map_err(|_| poisoned())? = evaluator;
    Ok(())
}

pub fn persist_config(state: &AppState) -> Result<(), RouteError> {
    let _io = state.config_io.lock().map_err(|_| poisoned())?;
    let rules = state.rules.lock().map_err(|_| poisoned())?.clone();
    let settings = state.settings.lock().map_err(|_| poisoned())?.clone();
    let config = AppConfig {
        version: CONFIG_VERSION,
        settings,
        rules,
    };
    state
        .repository
        .save_config(&config)
        .map_err(|e| RouteError::RuleEngine(e.to_string()))
}
