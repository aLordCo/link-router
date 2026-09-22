use tauri::State;
use url::Url;

use crate::core::domain::{
    AppConfig, AppSettings, BrowserProfile, RouteDecision, Rule, RuleEvaluation, RuleMatch,
    UrlRouteRequest, CONFIG_VERSION,
};
use crate::core::error::RouteError;
use crate::core::route_service::RouteEvaluator;
use crate::core::rules::RulesEngine;
use crate::core::sanitizer::sanitize_url;

use super::{commit_rules, effective_locale, persist_config, AppState};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<url::ParseError> for CommandError {
    fn from(e: url::ParseError) -> Self {
        Self::new("invalid_url", e.to_string())
    }
}

impl From<RouteError> for CommandError {
    fn from(e: RouteError) -> Self {
        Self::new("route_error", e.to_string())
    }
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        Self::new("default_handler", message)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for CommandError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::new("adapter_error", e.to_string())
    }
}

impl From<tauri::Error> for CommandError {
    fn from(e: tauri::Error) -> Self {
        Self::new("tauri_error", e.to_string())
    }
}

fn parse_url(raw: &str) -> Result<Url, CommandError> {
    raw.parse::<Url>().map_err(CommandError::from)
}

fn poisoned<T>(_: std::sync::PoisonError<T>) -> CommandError {
    CommandError::new("poisoned", "internal state lock poisoned")
}

fn sorted_rules(rules: &[Rule]) -> Vec<Rule> {
    let mut rules = rules.to_vec();
    rules.sort_by_key(|rule| rule.priority);
    rules
}

#[tauri::command]
pub fn evaluate_url(
    state: State<'_, AppState>,
    url: String,
    source_app: Option<String>,
) -> Result<RuleEvaluation, CommandError> {
    let parsed = parse_url(&url)?;
    let request = UrlRouteRequest {
        url: parsed.clone(),
        source_app,
    };

    let evaluator = state.evaluator.lock().map_err(poisoned)?;
    let matched = evaluator.match_rule(&request).map(|rule| RuleMatch {
        rule_id: rule.id.clone(),
        target_profile_id: rule.target_profile_id.clone(),
        pattern_kind: rule.pattern.kind_label(),
    });

    let cleaned = sanitize_url(&parsed);
    Ok(RuleEvaluation {
        url,
        cleaned_url: cleaned.to_string(),
        matched,
    })
}

#[tauri::command]
pub fn get_rules(state: State<'_, AppState>) -> Result<Vec<Rule>, CommandError> {
    let rules = state.rules.lock().map_err(poisoned)?;
    Ok(sorted_rules(&rules))
}

#[tauri::command]
pub fn create_rule(state: State<'_, AppState>, rule: Rule) -> Result<Rule, CommandError> {
    if rule.id.trim().is_empty() {
        return Err(CommandError::new("invalid_rule", "rule id cannot be empty"));
    }
    if rule.pattern.pattern_text().trim().is_empty() {
        return Err(CommandError::new(
            "invalid_rule",
            "rule pattern cannot be empty",
        ));
    }
    RulesEngine::validate_rule(&rule).map_err(CommandError::from)?;

    {
        let mut rules = state.rules.lock().map_err(poisoned)?;
        if rules.iter().any(|existing| existing.id == rule.id) {
            return Err(CommandError::new(
                "duplicate_rule",
                format!("rule '{}' already exists", rule.id),
            ));
        }
        rules.push(rule.clone());
    }
    commit_rules(&state).map_err(CommandError::from)?;
    Ok(rule)
}

#[tauri::command]
pub fn update_rule(state: State<'_, AppState>, rule: Rule) -> Result<Rule, CommandError> {
    RulesEngine::validate_rule(&rule).map_err(CommandError::from)?;

    {
        let mut rules = state.rules.lock().map_err(poisoned)?;
        let index = rules
            .iter()
            .position(|existing| existing.id == rule.id)
            .ok_or_else(|| {
                CommandError::new("rule_not_found", format!("rule '{}' not found", rule.id))
            })?;
        rules[index] = rule.clone();
    }
    commit_rules(&state).map_err(CommandError::from)?;
    Ok(rule)
}

#[tauri::command]
pub fn delete_rule(state: State<'_, AppState>, rule_id: String) -> Result<(), CommandError> {
    {
        let mut rules = state.rules.lock().map_err(poisoned)?;
        rules.retain(|existing| existing.id != rule_id);
    }
    commit_rules(&state).map_err(CommandError::from)?;
    Ok(())
}

#[tauri::command]
pub fn evaluate_url_route(
    state: State<'_, AppState>,
    url: String,
    source_app: Option<String>,
) -> Result<RouteDecision, CommandError> {
    let parsed = parse_url(&url)?;
    let request = UrlRouteRequest {
        url: parsed,
        source_app,
    };

    let profiles = state.detector.detect()?;
    let evaluator = state
        .evaluator
        .lock()
        .map_err(|_| CommandError::new("poisoned", "evaluator lock poisoned"))?;

    evaluator
        .evaluate(request, &profiles)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn open_in_browser(
    state: State<'_, AppState>,
    url: String,
    profile_id: String,
) -> Result<(), CommandError> {
    let parsed = parse_url(&url)?;
    let profiles = state.detector.detect()?;

    let profile = profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| {
            CommandError::new(
                "profile_not_found",
                format!("profile '{profile_id}' not found"),
            )
        })?;

    state.launcher.launch(profile, &parsed)?;
    Ok(())
}

#[tauri::command]
pub fn open_with_system_default(url: String) -> Result<(), CommandError> {
    let parsed = parse_url(&url)?;
    let target = parsed.as_str();

    #[cfg(target_os = "linux")]
    {
        return std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map(|_| ())
            .map_err(|e| CommandError::new("open_default_failed", e.to_string()));
    }

    #[cfg(target_os = "macos")]
    {
        return std::process::Command::new("open")
            .arg(target)
            .spawn()
            .map(|_| ())
            .map_err(|e| CommandError::new("open_default_failed", e.to_string()));
    }

    #[cfg(target_os = "windows")]
    {
        return std::process::Command::new("cmd")
            .args(["/C", "start", "", target])
            .spawn()
            .map(|_| ())
            .map_err(|e| CommandError::new("open_default_failed", e.to_string()));
    }

    #[allow(unreachable_code)]
    Err(CommandError::new(
        "unsupported_platform",
        "no default opener defined",
    ))
}

#[tauri::command]
pub fn list_browser_profiles(
    state: State<'_, AppState>,
) -> Result<Vec<BrowserProfile>, CommandError> {
    state.detector.detect().map_err(CommandError::from)
}

#[tauri::command]
pub fn set_default_browser(app: tauri::AppHandle) -> Result<(), CommandError> {
    super::associations::register_scheme(&app, "https").map_err(CommandError::from)?;
    super::associations::register_scheme(&app, "http").map_err(CommandError::from)
}

#[tauri::command]
pub fn register_scheme(app: tauri::AppHandle, scheme: String) -> Result<(), CommandError> {
    super::associations::register_scheme(&app, &scheme).map_err(CommandError::from)
}

#[tauri::command]
pub fn check_scheme(app: tauri::AppHandle, scheme: String) -> Result<bool, CommandError> {
    super::associations::check_scheme(&app, &scheme).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    state.settings.lock().map_err(poisoned).map(|s| s.clone())
}

fn validate_settings(settings: &AppSettings) -> Result<(), CommandError> {
    if !matches!(settings.locale.as_str(), "system" | "es" | "en")
        || !matches!(settings.theme.as_str(), "system" | "light" | "dark")
    {
        return Err(CommandError::new(
            "invalid_settings",
            "unsupported locale or theme",
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, CommandError> {
    let settings = state.settings.lock().map_err(poisoned)?.clone();
    let rules = state.rules.lock().map_err(poisoned)?.clone();
    Ok(AppConfig {
        version: CONFIG_VERSION,
        settings,
        rules: sorted_rules(&rules),
    })
}

#[tauri::command]
pub fn save_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<AppConfig, CommandError> {
    validate_settings(&config.settings)?;

    let mut seen_ids = std::collections::HashSet::new();
    for rule in &config.rules {
        if rule.id.trim().is_empty() {
            return Err(CommandError::new(
                "invalid_rule",
                "rule id cannot be empty",
            ));
        }
        if !seen_ids.insert(rule.id.clone()) {
            return Err(CommandError::new(
                "duplicate_rule",
                format!("rule '{}' appears more than once", rule.id),
            ));
        }
        RulesEngine::validate_rule(rule).map_err(CommandError::from)?;
    }
    let evaluator = RouteEvaluator::new(config.rules.clone()).map_err(CommandError::from)?;

    let (previous_settings, previous_rules) = {
        let mut settings = state.settings.lock().map_err(poisoned)?;
        let mut rules = state.rules.lock().map_err(poisoned)?;
        let previous = (settings.clone(), rules.clone());
        *settings = config.settings.clone();
        *rules = config.rules.clone();
        previous
    };

    if let Err(e) = persist_config(&state) {
        *state.settings.lock().map_err(poisoned)? = previous_settings;
        *state.rules.lock().map_err(poisoned)? = previous_rules;
        return Err(CommandError::new("config_persist_failed", e.to_string()));
    }

    *state.evaluator.lock().map_err(poisoned)? = evaluator;
    rebuild_tray(&app, &state)?;
    Ok(AppConfig {
        version: CONFIG_VERSION,
        settings: config.settings,
        rules: sorted_rules(&config.rules),
    })
}

#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), CommandError> {
    validate_settings(&settings)?;

    let previous = {
        let mut current = state.settings.lock().map_err(poisoned)?;
        let previous = current.clone();
        *current = settings.clone();
        previous
    };

    if let Err(e) = persist_config(&state) {
        *state.settings.lock().map_err(poisoned)? = previous;
        return Err(CommandError::new("settings_persist_failed", e.to_string()));
    }

    rebuild_tray(&app, &state)?;
    Ok(())
}

#[tauri::command]
pub fn system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| String::from("en"))
}

fn rebuild_tray(app: &tauri::AppHandle, state: &AppState) -> Result<(), CommandError> {
    let settings = state.settings.lock().map_err(poisoned)?;
    let locale = effective_locale(&settings);
    let menu = crate::build_tray_menu(app, locale).map_err(|e| {
        CommandError::new("tray_menu_error", format!("could not rebuild tray menu: {e}"))
    })?;
    let tooltip = crate::tray_labels(locale).3;
    if let Some(tray) = state.tray.lock().map_err(poisoned)?.as_ref() {
        tray.set_menu(Some(menu)).map_err(CommandError::from)?;
        tray.set_tooltip(Some(tooltip)).map_err(CommandError::from)?;
    }
    Ok(())
}
