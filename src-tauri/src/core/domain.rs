use std::path::PathBuf;

use url::Url;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProfile {
    pub id: String,
    pub name: String,
    pub browser_name: String,
    pub executable: PathBuf,
    pub profile_dir: Option<PathBuf>,
    pub icon: Option<String>,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UrlRouteRequest {
    pub url: Url,
    pub source_app: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RouteDecision {
    Launch {
        url: Url,
        profile_id: String,
    },
    Prompt {
        url: Url,
        candidates: Vec<BrowserProfile>,
    },
    None {
        url: Url,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RulePattern {
    Exact { pattern: String },
    Domain { pattern: String },
    Regex { pattern: String },
    Path { pattern: String },
    SourceApp { app_name: String },
}

impl RulePattern {
    pub fn kind_label(&self) -> &'static str {
        match self {
            RulePattern::Exact { .. } => "exact",
            RulePattern::Domain { .. } => "domain",
            RulePattern::Regex { .. } => "regex",
            RulePattern::Path { .. } => "path",
            RulePattern::SourceApp { .. } => "sourceApp",
        }
    }

    pub fn pattern_text(&self) -> &str {
        match self {
            RulePattern::Exact { pattern }
            | RulePattern::Domain { pattern }
            | RulePattern::Regex { pattern }
            | RulePattern::Path { pattern } => pattern,
            RulePattern::SourceApp { app_name } => app_name,
        }
    }
}

fn rule_enabled_default() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    #[serde(default)]
    pub priority: u32,
    #[serde(default = "rule_enabled_default")]
    pub enabled: bool,
    pub pattern: RulePattern,
    pub target_profile_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMatch {
    pub rule_id: String,
    pub target_profile_id: String,
    pub pattern_kind: &'static str,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleEvaluation {
    pub url: String,
    pub cleaned_url: String,
    pub matched: Option<RuleMatch>,
}
