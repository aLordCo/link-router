use super::domain::{BrowserProfile, RouteDecision, Rule, UrlRouteRequest};
use super::error::RouteError;
use super::rules::RulesEngine;
use super::sanitizer::sanitize_url;

pub struct RouteEvaluator {
    engine: RulesEngine,
}

impl RouteEvaluator {
    pub fn new(rules: Vec<Rule>) -> Result<Self, RouteError> {
        Ok(Self {
            engine: RulesEngine::new(rules)?,
        })
    }

    pub fn match_rule(&self, request: &UrlRouteRequest) -> Option<&Rule> {
        self.engine.find_match(request)
    }

    pub fn evaluate(
        &self,
        request: UrlRouteRequest,
        profiles: &[BrowserProfile],
    ) -> Result<RouteDecision, RouteError> {
        let cleaned = sanitize_url(&request.url);

        if let Some(rule) = self.engine.find_match(&request) {
            return Ok(RouteDecision::Launch {
                url: cleaned,
                profile_id: rule.target_profile_id.clone(),
            });
        }

        if profiles.is_empty() {
            return Ok(RouteDecision::None { url: cleaned });
        }

        Ok(RouteDecision::Prompt {
            url: cleaned,
            candidates: profiles.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    fn profile(id: &str) -> BrowserProfile {
        BrowserProfile {
            id: id.into(),
            name: id.into(),
            browser_name: "test".into(),
            executable: "/usr/bin/fake-browser".into(),
            profile_dir: None,
            icon: None,
            args: vec![],
        }
    }

    fn req(url: &str) -> UrlRouteRequest {
        UrlRouteRequest {
            url: Url::parse(url).unwrap(),
            source_app: None,
        }
    }

    #[test]
    fn launches_directly_when_rule_matches() {
        let evaluator = RouteEvaluator::new(vec![Rule {
            id: "r1".into(),
            priority: 0,
            enabled: true,
            pattern: super::super::domain::RulePattern::Domain {
                pattern: "company.com".into(),
            },
            target_profile_id: "brave-work".into(),
        }])
        .unwrap();

        let decision = evaluator
            .evaluate(
                req("https://intranet.company.com/deals"),
                &[profile("brave-work")],
            )
            .unwrap();

        assert!(
            matches!(decision, RouteDecision::Launch { profile_id, .. } if profile_id == "brave-work")
        );
    }

    #[test]
    fn prompts_when_no_rule_matches() {
        let evaluator = RouteEvaluator::new(vec![]).unwrap();
        let decision = evaluator
            .evaluate(req("https://example.com/x"), &[profile("chromium")])
            .unwrap();
        assert!(matches!(decision, RouteDecision::Prompt { .. }));
    }

    #[test]
    fn returns_none_when_no_browsers_available() {
        let evaluator = RouteEvaluator::new(vec![]).unwrap();
        let decision = evaluator
            .evaluate(req("https://example.com/x"), &[])
            .unwrap();
        assert!(matches!(decision, RouteDecision::None { .. }));
    }

    #[test]
    fn cleans_url_before_responding() {
        let evaluator = RouteEvaluator::new(vec![]).unwrap();
        let decision = evaluator
            .evaluate(
                req("https://example.com/p?id=1&utm_source=x&fbclid=y"),
                &[profile("chromium")],
            )
            .unwrap();
        match decision {
            RouteDecision::Prompt { url, .. } => {
                assert_eq!(url.as_str(), "https://example.com/p?id=1")
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }
}
