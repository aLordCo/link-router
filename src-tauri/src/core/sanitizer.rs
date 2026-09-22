use url::Url;

const TRACKING_PREFIXES: &[&str] = &["utm_", "ref_", "ga_", "_ga", "mc_", "mtm_"];

const TRACKING_EXACT: &[&str] = &[
    "fbclid", "gclid", "dclid", "msclkid", "yclid", "igshid", "si", "spm", "cmpid", "gclsrc",
];

fn is_tracking_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    if TRACKING_EXACT.contains(&lower.as_str()) {
        return true;
    }
    TRACKING_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

pub fn sanitize_url(url: &Url) -> Url {
    let mut sanitized = url.clone();

    let kept: Vec<(String, String)> = sanitized
        .query_pairs()
        .filter(|(key, _)| !is_tracking_key(key.as_ref()))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();

    sanitized.query_pairs_mut().clear().extend_pairs(kept);
    sanitized
}

#[allow(dead_code)]
pub fn is_short_url(url: &Url) -> bool {
    let domain = url.host_str().unwrap_or("").to_lowercase();
    domain == "t.co" || domain == "bit.ly" || domain == "tinyurl.com" || domain.ends_with("tiny.cc")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sanitize(raw: &str) -> String {
        sanitize_url(&Url::parse(raw).unwrap()).as_str().to_string()
    }

    #[test]
    fn removes_common_tracking_params() {
        assert_eq!(
            sanitize("https://example.com/?utm_source=newsletter&fbclid=xyz&id=123"),
            "https://example.com/?id=123"
        );
        assert_eq!(
            sanitize("https://example.com/a?gclid=abc&si=def&yclid=ghi&page=2"),
            "https://example.com/a?page=2"
        );
        assert_eq!(
            sanitize("https://example.com/a?mc_cid=1&ref_campaign=lead&ga_campaign=x&x=1"),
            "https://example.com/a?x=1"
        );
    }

    #[test]
    fn preserves_fragment_path_and_legit_params() {
        assert_eq!(
            sanitize("https://github.com/user/repo?tab=readme-ov-file&utm_source=x#section"),
            "https://github.com/user/repo?tab=readme-ov-file#section"
        );
    }

    #[test]
    fn keeps_order_of_legitimate_params() {
        assert_eq!(
            sanitize("https://example.com/p?b=2&utm_source=x&a=1&c=3"),
            "https://example.com/p?b=2&a=1&c=3"
        );
    }

    #[test]
    fn leaves_pristine_urls_untouched() {
        let raw = "https://github.com/tauri-apps/tauri?tab=readme-ov-file";
        assert_eq!(sanitize(raw), raw);
    }

    #[test]
    fn tracks_are_case_insensitive() {
        assert_eq!(
            sanitize("https://example.com/?UTM_Source=x&Fbclid=y&keep=1"),
            "https://example.com/?keep=1"
        );
    }

    #[test]
    fn detects_shorteners() {
        assert!(is_short_url(&Url::parse("https://t.co/xyz").unwrap()));
        assert!(is_short_url(&Url::parse("https://bit.ly/abc").unwrap()));
        assert!(!is_short_url(&Url::parse("https://github.com/x").unwrap()));
    }
}
