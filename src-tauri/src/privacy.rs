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

pub fn is_sensitive(text: &str) -> bool {
    SENSITIVE.is_match(text)
}
