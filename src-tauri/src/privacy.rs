use regex::RegexSet;
use std::sync::LazyLock;

static SENSITIVE: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b",
        r"\b(?:\d[ .()-]?){9,}\d\b",
        r"(?i)\b\d{1,6}\s+(?:[\p{L}]+\s+){1,4}(?:street|avenue|road|boulevard|lane|drive|st|ave|rd)\b",
        r"(?i)\b(diagnosis|diagnosed|prescription|medical|medication|patient|social security|ssn|bank account|credit card|routing number|password|api[_ -]?key|secret[_ -]?key)\b",
    ]).expect("valid built-in privacy rules")
});

// Conservative credential shapes: these do not require nearby words such as
// "password". A match cannot be overridden by approval or chat scope.
static SECRETS: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        r"\bsk-(?:proj-|ant-|svcacct-)?[A-Za-z0-9_-]{20,}",
        r"\bgh[pousr]_[A-Za-z0-9]{20,}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{20,}\b",
        r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b",
        r"-----BEGIN (?:[A-Z0-9]+ )*PRIVATE KEY-----",
        r"\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}",
        r"\bxox[baprs]-[A-Za-z0-9-]{10,}",
    ])
    .expect("valid built-in credential rules")
});

pub fn contains_secret(text: &str) -> bool {
    SECRETS.is_match(text)
        || text
            .split(|c: char| !c.is_ascii_alphanumeric() && !"_-/+=".contains(c))
            .any(|token| {
                // Catch opaque credentials without a known prefix. Length bounds and
                // mixed character classes avoid flagging normal prose and UUIDs.
                if !(32..=512).contains(&token.len())
                    || !token.bytes().any(|b| b.is_ascii_lowercase())
                    || !token.bytes().any(|b| b.is_ascii_uppercase())
                    || !token.bytes().any(|b| b.is_ascii_digit())
                {
                    return false;
                }
                let mut counts = [0usize; 128];
                for b in token.bytes() {
                    counts[b as usize] += 1;
                }
                let entropy: f64 = counts
                    .into_iter()
                    .filter(|n| *n > 0)
                    .map(|n| {
                        let p = n as f64 / token.len() as f64;
                        -p * p.log2()
                    })
                    .sum();
                entropy >= 4.5
            })
}

pub fn is_sensitive(text: &str) -> bool {
    contains_secret(text) || SENSITIVE.is_match(text)
}
