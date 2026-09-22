use regex::Regex;
use url::Url;

use super::domain::{Rule, RulePattern, UrlRouteRequest};
use super::error::RouteError;

enum Matcher {
    Exact { full_url: bool, expected: String },
    Domain { apex: String, subdomains_only: bool },
    Regex(Regex),
    Path { prefix: String, full_url: bool },
    SourceApp { app_name: String },
}

struct CompiledRule {
    source: Rule,
    matcher: Matcher,
}

impl CompiledRule {
    fn matches(&self, request: &UrlRouteRequest) -> bool {
        match &self.matcher {
            Matcher::Exact { full_url, expected } => {
                if *full_url {
                    request.url.as_str().eq_ignore_ascii_case(expected)
                } else {
                    host(&request.url)
                        .map(|h| h.eq_ignore_ascii_case(expected))
                        .unwrap_or(false)
                }
            }
            Matcher::Domain {
                apex,
                subdomains_only,
            } => match host(&request.url) {
                Some(h) => {
                    let suffix = format!(".{apex}");
                    if *subdomains_only {
                        h != *apex && h.ends_with(&suffix)
                    } else {
                        h == *apex || h.ends_with(&suffix)
                    }
                }
                None => false,
            },
            Matcher::Regex(regex) => regex.is_match(request.url.as_str()),
            Matcher::Path { prefix, full_url } => {
                path_candidate(&request.url, *full_url).starts_with(prefix.as_str())
            }
            Matcher::SourceApp { app_name } => request
                .source_app
                .as_deref()
                .map(|source| source.to_lowercase() == *app_name)
                .unwrap_or(false),
        }
    }
}

fn host(url: &Url) -> Option<String> {
    url.host_str().map(str::to_lowercase)
}

fn path_candidate(url: &Url, full_url: bool) -> String {
    if full_url {
        url.as_str().to_string()
    } else {
        format!(
            "{}/{}",
            host(url).unwrap_or_default(),
            url.path().trim_start_matches('/')
        )
    }
}

fn compile_matcher(pattern: &RulePattern) -> Result<Matcher, RouteError> {
    match pattern {
        RulePattern::Exact { pattern } => {
            let trimmed = pattern.trim();
            if trimmed.is_empty() {
                return Err(RouteError::RuleEngine(
                    "exact pattern cannot be empty".into(),
                ));
            }
            Ok(Matcher::Exact {
                full_url: trimmed.contains("://"),
                expected: trimmed.to_string(),
            })
        }
        RulePattern::Domain { pattern } => {
            let trimmed = pattern.trim();
            let subdomains_only = trimmed.starts_with("*.");
            let apex = if subdomains_only {
                &trimmed[2..]
            } else {
                trimmed
            };
            if apex.is_empty() || apex.contains("://") || apex.contains('/') {
                return Err(RouteError::RuleEngine(
                    "domain pattern must be a host like 'github.com' or '*.github.com'".into(),
                ));
            }
            Ok(Matcher::Domain {
                apex: apex.to_lowercase(),
                subdomains_only,
            })
        }
        RulePattern::Regex { pattern } => {
            let trimmed = pattern.trim();
            if trimmed.is_empty() {
                return Err(RouteError::RuleEngine(
                    "regex pattern cannot be empty".into(),
                ));
            }
            Regex::new(trimmed)
                .map(Matcher::Regex)
                .map_err(|e| RouteError::RuleEngine(e.to_string()))
        }
        RulePattern::Path { pattern } => {
            let mut prefix = pattern.trim().to_string();
            if prefix.ends_with('*') {
                prefix.pop();
            }
            prefix = prefix.trim_end().to_string();
            if prefix.is_empty() {
                return Err(RouteError::RuleEngine("path prefix cannot be empty".into()));
            }
            Ok(Matcher::Path {
                full_url: prefix.contains("://"),
                prefix,
            })
        }
        RulePattern::SourceApp { app_name } => Ok(Matcher::SourceApp {
            app_name: app_name.trim().to_lowercase(),
        }),
    }
}

pub struct RulesEngine {
    compiled: Vec<CompiledRule>,
}

impl RulesEngine {
    pub fn new(rules: Vec<Rule>) -> Result<Self, RouteError> {
        let mut rules = rules;
        rules.sort_by_key(|rule| rule.priority);

        let mut compiled = Vec::with_capacity(rules.len());
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let matcher = compile_matcher(&rule.pattern)?;
            compiled.push(CompiledRule {
                source: rule,
                matcher,
            });
        }
        Ok(Self { compiled })
    }

    pub fn validate_rule(rule: &Rule) -> Result<(), RouteError> {
        compile_matcher(&rule.pattern).map(|_| ())
    }

    pub fn find_match(&self, request: &UrlRouteRequest) -> Option<&Rule> {
        self.compiled
            .iter()
            .find(|compiled| compiled.matches(request))
            .map(|compiled| &compiled.source)
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    fn request(url: &str, source_app: Option<&str>) -> UrlRouteRequest {
        UrlRouteRequest {
            url: Url::parse(url).unwrap(),
            source_app: source_app.map(str::to_string),
        }
    }

    fn rule(id: &str, priority: u32, pattern: RulePattern, target: &str) -> Rule {
        Rule {
            id: id.into(),
            priority,
            enabled: true,
            pattern,
            target_profile_id: target.into(),
        }
    }

    fn domain(pattern: &str) -> RulePattern {
        RulePattern::Domain {
            pattern: pattern.into(),
        }
    }

    #[test]
    fn exact_host_matches_case_insensitively() {
        let engine = RulesEngine::new(vec![rule(
            "e1",
            0,
            RulePattern::Exact {
                pattern: "Github.com".into(),
            },
            "chrome",
        )])
        .unwrap();
        assert_eq!(
            engine
                .find_match(&request("https://github.com/tauri-apps/tauri", None))
                .unwrap()
                .target_profile_id,
            "chrome"
        );
        assert!(engine
            .find_match(&request("https://example.com/github.com", None))
            .is_none());
    }

    #[test]
    fn exact_full_url_matches() {
        let engine = RulesEngine::new(vec![rule(
            "e2",
            0,
            RulePattern::Exact {
                pattern: "https://meet.google.com/abc-def".into(),
            },
            "chrome",
        )])
        .unwrap();
        assert!(engine
            .find_match(&request("https://meet.google.com/abc-def", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://meet.google.com/abc-def/other", None))
            .is_none());
    }

    #[test]
    fn domain_apex_matches_subdomains() {
        let engine =
            RulesEngine::new(vec![rule("d1", 0, domain("github.com"), "chrome-work")]).unwrap();
        assert!(engine
            .find_match(&request("https://github.com/x", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://www.github.com/x", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://hey.github.com/x", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://example.com/x", None))
            .is_none());
        assert!(engine
            .find_match(&request("https://notgithub.com/x", None))
            .is_none());
    }

    #[test]
    fn domain_wildcard_excludes_apex() {
        let engine =
            RulesEngine::new(vec![rule("d2", 0, domain("*.github.com"), "brave")]).unwrap();
        assert!(engine
            .find_match(&request("https://docs.github.com/x", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://github.com/x", None))
            .is_none());
    }

    #[test]
    fn matches_regex_over_full_url() {
        let engine = RulesEngine::new(vec![rule(
            "r1",
            0,
            RulePattern::Regex {
                pattern: r"^https://.*\.notion\.site/.+".into(),
            },
            "arc",
        )])
        .unwrap();
        assert!(engine
            .find_match(&request("https://team.notion.site/Notes", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://example.com/Notion", None))
            .is_none());
    }

    #[test]
    fn path_prefix_matches_scheme_and_host_forms() {
        let engine = RulesEngine::new(vec![
            rule(
                "p1",
                0,
                RulePattern::Path {
                    pattern: "https://meet.google.com/*".into(),
                },
                "chrome",
            ),
            rule(
                "p2",
                0,
                RulePattern::Path {
                    pattern: "intranet.company.com/deals".into(),
                },
                "brave",
            ),
        ])
        .unwrap();
        assert!(engine
            .find_match(&request("https://meet.google.com/abc?token=1", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://meet.google.com", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://meet2.google.com/abc", None))
            .is_none());
        assert!(engine
            .find_match(&request("https://intranet.company.com/deals/123", None))
            .is_some());
        assert!(engine
            .find_match(&request("https://intranet.company.com/home", None))
            .is_none());
    }

    #[test]
    fn matches_source_app_case_insensitively() {
        let engine = RulesEngine::new(vec![rule(
            "s1",
            0,
            RulePattern::SourceApp {
                app_name: "slack".into(),
            },
            "brave-work",
        )])
        .unwrap();
        let matched = engine
            .find_match(&request("https://github.com/x", Some("Slack")))
            .unwrap();
        assert_eq!(matched.target_profile_id, "brave-work");
        assert!(engine
            .find_match(&request("https://github.com/x", Some("Discord")))
            .is_none());
    }

    #[test]
    fn higher_priority_wins_overlapping_rules() {
        let low = rule("low", 5, domain("github.com"), "fallback");
        let high = rule(
            "high",
            1,
            RulePattern::Path {
                pattern: "https://github.com/tauri-apps/*".into(),
            },
            "preferred",
        );
        let engine = RulesEngine::new(vec![low, high]).unwrap();

        let matched = engine
            .find_match(&request("https://github.com/tauri-apps/tauri", None))
            .unwrap();
        assert_eq!(matched.id, "high");
        assert_eq!(matched.target_profile_id, "preferred");

        let other = engine
            .find_match(&request("https://github.com/other/repo", None))
            .unwrap();
        assert_eq!(other.id, "low");
    }

    #[test]
    fn equal_priority_ties_break_in_favor_of_first_inserted() {
        let first = rule("first", 3, domain("github.com"), "a");
        let second = rule("second", 3, domain("github.com"), "b");
        let engine = RulesEngine::new(vec![first, second]).unwrap();
        let matched = engine
            .find_match(&request("https://github.com/x", None))
            .unwrap();
        assert_eq!(matched.id, "first");
    }

    #[test]
    fn disabled_rules_are_skipped() {
        let engine = RulesEngine::new(vec![Rule {
            id: "off".into(),
            priority: 0,
            enabled: false,
            pattern: domain("github.com"),
            target_profile_id: "brave".into(),
        }])
        .unwrap();
        assert!(engine
            .find_match(&request("https://github.com/x", None))
            .is_none());
    }

    #[test]
    fn rejects_invalid_patterns() {
        assert!(RulesEngine::new(vec![rule(
            "bad",
            0,
            RulePattern::Regex {
                pattern: "(".into(),
            },
            "x"
        )])
        .is_err());
        assert!(RulesEngine::new(vec![rule("bad", 0, domain("http://x/y"), "x")]).is_err());
        assert!(RulesEngine::new(vec![rule(
            "bad",
            0,
            RulePattern::Exact { pattern: "".into() },
            "x"
        )])
        .is_err());
    }
}
