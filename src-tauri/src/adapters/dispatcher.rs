use tauri::{AppHandle, Emitter, Manager};

use crate::core::domain::{RouteDecision, UrlRouteRequest};

use super::AppState;

pub const INCOMING_URL_EVENT: &str = "dispatch://incoming-url";

pub fn extract_url_arg(args: &[String]) -> Option<String> {
    args.iter().find_map(|arg| {
        let candidate = arg.trim();
        let parsed = url::Url::parse(candidate).ok()?;
        match parsed.scheme() {
            "http" | "https" => Some(candidate.to_string()),
            _ => None,
        }
    })
}

pub fn handle_incoming_url(
    app: &AppHandle,
    raw: &str,
    source_app: Option<&str>,
) -> Result<(), String> {
    let Some(state) = app.try_state::<AppState>() else {
        log::warn!("dispatch: AppState no está listo aún");
        return Ok(());
    };

    if !state.should_handle(raw) {
        return Ok(());
    }

    let parsed: url::Url = raw
        .trim()
        .parse()
        .map_err(|e| format!("url inválida en dispatcher: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Ok(());
    }

    let request = UrlRouteRequest {
        url: parsed,
        source_app: source_app.map(str::to_string),
    };

    let profiles = state
        .detector
        .detect()
        .map_err(|e| format!("detección de perfiles falló: {e}"))?;

    let decision = {
        let evaluator = state
            .evaluator
            .lock()
            .map_err(|_| "evaluator lock poisoned".to_string())?;
        evaluator
            .evaluate(request, &profiles)
            .map_err(|e| e.to_string())?
    };

    match decision {
        RouteDecision::Launch { url, profile_id } => {
            let Some(profile) = profiles.iter().find(|p| p.id == profile_id) else {
                log::warn!("dispatch: la regla apunta a '{profile_id}' pero no fue detectado");
                return show_prompt(app, url.as_str(), source_app);
            };
            state
                .launcher
                .launch(profile, &url)
                .map_err(|e| format!("lanzamiento del navegador falló: {e}"))
        }
        RouteDecision::Prompt { url, .. } | RouteDecision::None { url } => {
            show_prompt(app, url.as_str(), source_app)
        }
    }
}

fn show_prompt(app: &AppHandle, url: &str, source_app: Option<&str>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(true);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }

    app.emit(
        INCOMING_URL_EVENT,
        serde_json::json!({ "url": url, "sourceApp": source_app }),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_http_and_https_args() {
        let args: Vec<String> = vec![
            "/usr/bin/linkrouter".into(),
            "https://github.com/octocat/Hello-World?utm_source=x".into(),
        ];
        assert_eq!(
            extract_url_arg(&args).as_deref(),
            Some("https://github.com/octocat/Hello-World?utm_source=x")
        );
    }

    #[test]
    fn ignores_non_http_args() {
        let args: Vec<String> = vec!["--flag".into(), "mailto:alice@example.com".into()];
        assert_eq!(extract_url_arg(&args), None);
    }

    #[test]
    fn picks_the_first_real_url_amid_switches() {
        let args: Vec<String> = vec![
            "linkrouter".into(),
            "--no-sandbox".into(),
            "http://intranet.company.com/deals".into(),
        ];
        assert_eq!(
            extract_url_arg(&args).as_deref(),
            Some("http://intranet.company.com/deals")
        );
    }
}