use regex::RegexSet;
use std::sync::LazyLock;

static SENSITIVE: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b",
        // Formatted North American numbers, international + prefixes, or explicit labels.
        // Bare epoch timestamps / identifiers are not evidence of a phone number.
        r"(?:\b\d{3}[ .-]|\(\d{3}\)[ ]?)\d{3}[ .-]?\d{4}\b",
        r"\+\d(?:[ ().-]?\d){7,14}\b",
        r"(?i)\b(?:phone|telephone|tel|mobile|fax|whatsapp|call)\s*(?:number\s*)?(?:is\s*)?[:=]?\s*\d{9,15}\b",
        r"(?i)\b(?:call|text|reach|contact)\s+(?:me|us)\s+(?:at|on)\s*[:=]?\s*\d{9,15}\b",
        r"(?i)\b\d{1,6}\s+(?:[\p{L}]+\s+){1,4}(?:street|avenue|road|boulevard|lane|drive|st|ave|rd)\b",
        r"(?i)\b(diagnosis|diagnosed|prescription|medical|medication|patient|social security|ssn|bank account|credit card|routing number)\b",
        r#"(?i)\b(?:password|passwd|api[_ -]?key|secret[_ -]?key)\b["']?\s*[:=]\s*["']?[^\s"']{6,}"#,
    ]).expect("valid built-in privacy rules")
});

// Conservative credential shapes: these do not require nearby words such as
// "password". A match cannot be overridden by approval or chat scope.
static SECRETS: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        // Contextual assignments include short passcodes and ordinary words.
        r#"(?i)\b(?:password|passwd|passcode|pin|api[_ -]?key|secret[_ -]?key)\b["']?\s*(?:[:=]|is\b)\s*["']?[^\s"']+"#,
        // Bare assignments with a digit, while avoiding normal technical prose.
        r#"(?i)\b(?:password|passwd|passcode|pin)\s+["']?[^\s"']*\d[^\s"']*"#,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_and_technical_discussion_are_not_personal_data_by_themselves() {
        for text in [
            "created_at: 1795132800",
            "updated_at: 1795132800123",
            "1795132800.123456",
            "Build identifier 12345678901234567890",
            "2026-11-19 17:04:49",
            "How do I hash a password in Django?",
            "Store an API key in an environment variable.",
        ] {
            assert!(!is_sensitive(text), "false flag: {text}");
        }
    }

    #[test]
    fn phone_shapes_and_assigned_credentials_remain_restricted() {
        for text in [
            "514-555-1234",
            "(514) 555-1234",
            "514.555.1234",
            "514 555 1234",
            "+1 514 555 1234",
            "+442079460958",
            "+33 1 23 45 67 89",
            "phone: 5145551234",
            "mobile number is 447911123456",
            "password = synthetic-secret",
            "my wifi password is fictional-lilac",
            "password Demo42!",
            "PIN: 42",
            "passcode is 1234",
            "call me at 2025550147",
            "text us on 2025550147",
            r#"{"api_key": "synthetic-secret"}"#,
        ] {
            assert!(is_sensitive(text), "missed: {text}");
        }
    }
}
