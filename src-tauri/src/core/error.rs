use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouteError {
    #[error("malformed url: {0}")]
    #[allow(dead_code)]
    InvalidUrl(String),
    #[error("rule engine failure: {0}")]
    RuleEngine(String),
}
