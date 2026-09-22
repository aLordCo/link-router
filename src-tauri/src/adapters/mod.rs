pub mod associations;
pub mod commands;
pub mod detectors;
pub mod dispatcher;
pub mod launchers;
pub mod persistence;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::AppHandle;

use crate::core::domain::Rule;
use crate::core::error::RouteError;
use crate::core::route_service::RouteEvaluator;
use crate::ports::{BrowserDetectorPort, ConfigRepositoryPort, UrlLauncherPort};

pub struct AppState {
    pub detector: Box<dyn BrowserDetectorPort>,
    pub launcher: Box<dyn UrlLauncherPort>,
    pub repository: Box<dyn ConfigRepositoryPort>,
    pub rules: Mutex<Vec<Rule>>,
    pub evaluator: Mutex<RouteEvaluator>,
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

fn initial_rules(repository: &dyn ConfigRepositoryPort) -> Vec<Rule> {
    match repository.load_rules() {
        Ok(rules) => rules,
        Err(e) => {
            log::warn!("could not load rules from repository: {e}");
            Vec::new()
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
    let rules = initial_rules(&repository);
    let evaluator = initial_evaluator(rules.clone());

    AppState {
        detector: Box::new(detectors::system_detector()),
        launcher: Box::new(launchers::ProcessLauncher::new()),
        repository: Box::new(repository),
        rules: Mutex::new(rules),
        evaluator: Mutex::new(evaluator),
        last_url: Mutex::new(None),
    }
}

#[allow(dead_code)]
pub fn reload_rules(state: &AppState) -> Result<(), RouteError> {
    let rules = state
        .repository
        .load_rules()
        .map_err(|e| RouteError::RuleEngine(e.to_string()))?;
    *state.rules.lock().map_err(|_| poisoned())? = rules.clone();
    let evaluator = initial_evaluator(rules);
    *state.evaluator.lock().map_err(|_| poisoned())? = evaluator;
    Ok(())
}

pub fn commit_rules(state: &AppState) -> Result<(), RouteError> {
    let rules = state.rules.lock().map_err(|_| poisoned())?.clone();
    let evaluator = RouteEvaluator::new(rules.clone())?;
    state
        .repository
        .save_rules(&rules)
        .map_err(|e| RouteError::RuleEngine(e.to_string()))?;
    *state.evaluator.lock().map_err(|_| poisoned())? = evaluator;
    Ok(())
}
