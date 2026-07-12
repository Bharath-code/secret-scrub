//! High-precision built-in detectors.
//!
//! Overlap precedence: earlier start wins; if starts equal, longer match wins;
//! if still tied, higher specificity (lower rank number) wins.

use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub start: usize,
    pub end: usize,
    pub detector_type: &'static str,
    /// Lower is more specific (provider tokens beat generic/email/ip).
    pub specificity: u8,
    pub value: String,
}

struct Detector {
    detector_type: &'static str,
    specificity: u8,
    regex: Regex,
}

fn detectors() -> &'static [Detector] {
    static DETECTORS: OnceLock<Vec<Detector>> = OnceLock::new();
    DETECTORS.get_or_init(|| {
        // Patterns are intentionally high-signal (precision-first).
        // Use only synthetic values in fixtures — never real secrets.
        vec![
            det(
                "AWS_ACCESS_KEY",
                10,
                r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b",
            ),
            det(
                "GITHUB_TOKEN",
                10,
                r"\b(?:ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{20,})\b",
            ),
            det(
                "STRIPE_SECRET",
                10,
                r"\bsk_(?:live|test)_[A-Za-z0-9]{16,}\b",
            ),
            // No hyphens in the body (rejects slug-like names) and digits
            // required via post-filter in find_candidates.
            det(
                "OPENAI_API_KEY",
                10,
                r"\bsk-(?:proj-)?[A-Za-z0-9_]{20,}\b",
            ),
            det(
                "JWT",
                20,
                r"\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b",
            ),
            // Generic env/header secrets: name=value with secret-ish names (capture value only).
            det(
                "GENERIC_SECRET",
                40,
                r#"(?i)\b(?:api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|private[_-]?key|password|passwd|client[_-]?secret)\b\s*[=:]\s*(?:"([^"]{8,})"|'([^']{8,})'|([^\s#'"]{8,}))"#,
            ),
            det(
                "EMAIL",
                50,
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b",
            ),
            det(
                "IP_ADDRESS",
                60,
                r"\b(?:(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\b",
            ),
        ]
    })
}

fn det(detector_type: &'static str, specificity: u8, pattern: &str) -> Detector {
    Detector {
        detector_type,
        specificity,
        regex: Regex::new(pattern).expect("built-in detector regex must compile"),
    }
}

/// Find all non-overlapping candidates after resolving overlaps.
pub fn find_candidates(text: &str) -> Vec<Candidate> {
    let mut raw: Vec<Candidate> = Vec::new();

    for d in detectors() {
        for caps in d.regex.captures_iter(text) {
            // Multiple alternative capture groups (e.g. quoted vs bare value
            // branches): first non-empty group wins.
            let (start, end, value) = if let Some(m) =
                (1..caps.len()).find_map(|i| caps.get(i))
            {
                (m.start(), m.end(), m.as_str().to_string())
            } else if let Some(m) = caps.get(0) {
                (m.start(), m.end(), m.as_str().to_string())
            } else {
                continue;
            };

            // Entropy floor: real OpenAI keys always carry digits; slug-like
            // identifiers (sk-formatting-helper) usually don't.
            if d.detector_type == "OPENAI_API_KEY"
                && value.chars().filter(char::is_ascii_digit).count() < 2
            {
                continue;
            }

            raw.push(Candidate {
                start,
                end,
                detector_type: d.detector_type,
                specificity: d.specificity,
                value,
            });
        }
    }

    resolve_overlaps(raw)
}

/// Sort and greedily keep non-overlapping matches by precedence.
fn resolve_overlaps(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    candidates.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| (b.end - b.start).cmp(&(a.end - a.start)))
            .then_with(|| a.specificity.cmp(&b.specificity))
            .then_with(|| a.detector_type.cmp(b.detector_type))
    });

    let mut accepted: Vec<Candidate> = Vec::new();
    for c in candidates {
        let overlaps = accepted
            .iter()
            .any(|a| c.start < a.end && c.end > a.start);
        if !overlaps {
            accepted.push(c);
        }
    }
    accepted.sort_by_key(|c| c.start);
    accepted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_aws_access_key() {
        let text = "key=AKIAIOSFODNN7EXAMPLE rest";
        let c = find_candidates(text);
        assert!(c.iter().any(|x| x.detector_type == "AWS_ACCESS_KEY"));
    }

    #[test]
    fn slug_like_sk_name_not_openai() {
        assert!(find_candidates("tool=sk-formatting-helper-utils-v2 done").is_empty());
    }

    #[test]
    fn openai_key_detected() {
        let c = find_candidates("openai=sk-proj-abcdefghijklmnopqrstuvwxyz012345");
        assert!(c.iter().any(|x| x.detector_type == "OPENAI_API_KEY"));
    }

    #[test]
    fn stripe_not_openai() {
        let text = "sk_test_51HaExampleStripeKey99";
        let c = find_candidates(text);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].detector_type, "STRIPE_SECRET");
    }

    #[test]
    fn generic_secret_quoted_double() {
        let text = r#"password="supersecret1""#;
        let c = find_candidates(text);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].detector_type, "GENERIC_SECRET");
        assert_eq!(c[0].value, "supersecret1");
        // Quotes fall outside the match span, so they survive replacement.
        assert_eq!(&text[c[0].start..c[0].end], "supersecret1");
    }

    #[test]
    fn generic_secret_quoted_single() {
        let text = "api_key: 'abcdefgh1234'";
        let c = find_candidates(text);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].value, "abcdefgh1234");
    }

    #[test]
    fn generic_secret_bare_still_works() {
        let text = "password=hunter2secret99 rest";
        let c = find_candidates(text);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].value, "hunter2secret99");
    }

    #[test]
    fn generic_secret_short_quoted_not_matched() {
        assert!(find_candidates(r#"password="short""#).is_empty());
    }

    #[test]
    fn generic_secret_quoted_alternation_yields_single_match() {
        // Alternation branches are mutually exclusive at a given start
        // position, so a quoted value is never counted twice.
        let text = r#"password="supersecret1" trailing text"#;
        assert_eq!(find_candidates(text).len(), 1);
    }
}
