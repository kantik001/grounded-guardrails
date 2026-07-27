//! Regex-based PII detection for streaming / post-hoc text checks.
//!
//! Patterns are compiled once via [`std::sync::LazyLock`]. Prefer [`PiiDetector::contains_pii`]
//! on the hot path when you only need a boolean gate.

use regex::Regex;
use std::sync::LazyLock;

static RE_EMAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}\b").expect("email regex")
});

// North America + RU-style mobiles; require enough digits to avoid short order IDs.
static RE_PHONE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        (?:
            \+\s*1[\s.\-]*\(\s*\d{3}\s*\)[\s.\-]*\d{3}[\s.\-]*\d{4}
          | \+\s*7[\s.\-]*\d{3}[\s.\-]*\d{3}[\s.\-]*\d{2}[\s.\-]*\d{2}
          | \b\d{3}[.\-]\d{3}[.\-]\d{4}\b
          | \b\(\d{3}\)\s*\d{3}[\s.\-]*\d{4}\b
        )",
    )
    .expect("phone regex")
});

static RE_SSN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").expect("ssn regex"));

static RE_CREDIT_CARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        \b(?:
            \d{4}[\s\-]\d{4}[\s\-]\d{4}[\s\-]\d{4}
          | \d{16}
        )\b",
    )
    .expect("credit card regex")
});

/// Kind of personally identifiable information detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiiType {
    Email,
    Phone,
    Ssn,
    CreditCard,
}

impl PiiType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Ssn => "ssn",
            Self::CreditCard => "credit_card",
        }
    }
}

impl std::fmt::Display for PiiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single PII span with a redacted display form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiiMatch {
    pub pii_type: PiiType,
    pub start: usize,
    pub end: usize,
    /// Redacted form, e.g. `j***@example.com`.
    pub masked: String,
}

/// Regex PII detector. Cheap to clone (shared compiled patterns).
#[derive(Debug, Clone, Default)]
pub struct PiiDetector;

impl PiiDetector {
    /// Create a detector (patterns are process-global).
    pub fn new() -> Self {
        Self
    }

    /// Return every PII span in `text` (byte offsets into `text`).
    pub fn detect(&self, text: &str) -> Vec<PiiMatch> {
        let mut out = Vec::new();
        collect(text, &RE_EMAIL, PiiType::Email, mask_email, &mut out);
        collect(text, &RE_PHONE, PiiType::Phone, mask_phone, &mut out);
        collect(text, &RE_SSN, PiiType::Ssn, mask_ssn, &mut out);
        collect(
            text,
            &RE_CREDIT_CARD,
            PiiType::CreditCard,
            mask_credit_card,
            &mut out,
        );
        out.sort_by_key(|m| (m.start, m.end));
        out
    }

    /// Fast path: true if any PII pattern matches.
    pub fn contains_pii(&self, text: &str) -> bool {
        RE_EMAIL.is_match(text)
            || RE_PHONE.is_match(text)
            || RE_SSN.is_match(text)
            || RE_CREDIT_CARD.is_match(text)
    }
}

fn collect(
    text: &str,
    re: &Regex,
    pii_type: PiiType,
    mask: fn(&str) -> String,
    out: &mut Vec<PiiMatch>,
) {
    for m in re.find_iter(text) {
        out.push(PiiMatch {
            pii_type,
            start: m.start(),
            end: m.end(),
            masked: mask(m.as_str()),
        });
    }
}

fn mask_email(raw: &str) -> String {
    let Some((local, domain)) = raw.split_once('@') else {
        return "***".to_string();
    };
    let first = local.chars().next().unwrap_or('*');
    format!("{first}***@{domain}")
}

fn mask_phone(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 4 {
        return "***".to_string();
    }
    let last4 = &digits[digits.len() - 4..];
    format!("***-***-{last4}")
}

fn mask_ssn(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        format!("***-**-{}", &digits[digits.len() - 4..])
    } else {
        "***-**-****".to_string()
    }
}

fn mask_credit_card(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        format!("**** **** **** {}", &digits[digits.len() - 4..])
    } else {
        "**** **** **** ****".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_emails() {
        let d = PiiDetector::new();
        let m = d.detect("Contact user@example.com please");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].pii_type, PiiType::Email);
        assert_eq!(m[0].masked, "u***@example.com");

        let m2 = d.detect("mail first.last+tag@sub.domain.co.uk ok");
        assert!(m2.iter().any(|x| x.pii_type == PiiType::Email));
    }

    #[test]
    fn detects_phones() {
        let d = PiiDetector::new();
        assert!(d.contains_pii("Call +1 (555) 123-4567 now"));
        assert!(d.contains_pii("Call 555.123.4567 now"));
        assert!(d.contains_pii("Call +7 999 123-45-67 now"));
    }

    #[test]
    fn detects_ssn_and_card() {
        let d = PiiDetector::new();
        let ssn = d.detect("SSN 123-45-6789");
        assert_eq!(ssn.len(), 1);
        assert_eq!(ssn[0].pii_type, PiiType::Ssn);
        assert_eq!(ssn[0].masked, "***-**-6789");

        let card = d.detect("card 4111 1111 1111 1111");
        assert_eq!(card.len(), 1);
        assert_eq!(card[0].pii_type, PiiType::CreditCard);
        assert!(d.contains_pii("4111111111111111"));
    }

    #[test]
    fn false_positives_versions_and_order_ids() {
        let d = PiiDetector::new();
        assert!(!d.contains_pii("version 1.2.3 released"));
        assert!(!d.contains_pii("order #123-456 shipped"));
        assert!(!d.contains_pii("build 2024.07.27"));
    }
}
