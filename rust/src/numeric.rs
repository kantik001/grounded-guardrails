//! Numeric extraction and context verification.
//!
//! This is the **canonical** numeric algorithm for the Grounded ecosystem.
//! Go gRPC and Python clients should call this logic (via FFI/gRPC), not fork it.
//!
//! Tolerance matches Grounded LLM Spec practice: absolute Δ ≤ `0.01` by default.

use regex::Regex;
use std::sync::LazyLock;

/// Default absolute tolerance used by Grounded LLM verify.
pub const DEFAULT_TOLERANCE: f64 = 0.01;

static RE_NUMERIC: LazyLock<Regex> = LazyLock::new(|| {
    // US thousands, EU thousands, plain int/float, optional % and ±.
    Regex::new(
        r"(?x)
        [±+\-]?
        (?:
            \d{1,3}(?:,\d{3})+(?:\.\d+)?
          | \d{1,3}(?:\.\d{3})+(?:,\d+)?
          | \d+(?:[.,]\d+)?
        )
        (?:\s*%)?",
    )
    .expect("numeric regex")
});

static RE_MULTIPLIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(млн|миллион(?:а|ов)?|million|тыс\.?|тысяч[аи]?|thousand|млрд|миллиард(?:а|ов)?|billion)\b")
        .expect("multiplier regex")
});

/// A numeric span extracted from text.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericValue {
    pub value: f64,
    pub start: usize,
    pub end: usize,
    /// Original matched lexeme (e.g. `14,000,000` or `14.5%`).
    pub raw: String,
}

/// Extract all numbers from `text` (integers, floats, percentages, grouped digits).
pub fn extract_numerics(text: &str) -> Vec<NumericValue> {
    let mut out = Vec::new();
    for m in RE_NUMERIC.find_iter(text) {
        let raw = m.as_str();
        let Some(mut value) = parse_numeric_lexeme(raw) else {
            continue;
        };
        let mut end = m.end();
        if let Some(cap) = RE_MULTIPLIER.find(&text[m.end()..]) {
            value *= multiplier_factor(cap.as_str());
            end = m.end() + cap.end();
        }
        out.push(NumericValue {
            value,
            start: m.start(),
            end,
            raw: text[m.start()..end].to_string(),
        });
    }
    out
}

/// True if `value` appears in `context` within `tolerance` (absolute).
pub fn verify_numeric(value: f64, context: &[NumericValue], tolerance: f64) -> bool {
    context.iter().any(|c| (c.value - value).abs() <= tolerance)
}

/// Verify every number in `answer` against numbers in `context_text`.
///
/// Empty answer numbers → pass (same semantics as Grounded LLM Go verify).
pub fn verify_answer_against_context(answer: &str, context_text: &str, tolerance: f64) -> bool {
    let answer_nums = extract_numerics(answer);
    if answer_nums.is_empty() {
        return true;
    }
    let context_nums = extract_numerics(context_text);
    answer_nums
        .iter()
        .all(|n| verify_numeric(n.value, &context_nums, tolerance))
}

/// Numbers in `answer` that have no match in `context` within `tolerance`.
pub fn unmatched_numerics(answer: &str, context_text: &str, tolerance: f64) -> Vec<f64> {
    let context_nums = extract_numerics(context_text);
    extract_numerics(answer)
        .into_iter()
        .map(|n| n.value)
        .filter(|v| !verify_numeric(*v, &context_nums, tolerance))
        .collect()
}

fn parse_numeric_lexeme(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    // ±0.01 → 0.01 (magnitude); plain -3 → -3.
    let (negative, body) = if trimmed.starts_with('±') {
        (false, trimmed.trim_start_matches('±'))
    } else if trimmed.starts_with('+') {
        (false, trimmed.trim_start_matches('+'))
    } else if trimmed.starts_with('-') {
        (true, trimmed.trim_start_matches('-'))
    } else {
        (false, trimmed)
    };
    let core = body.trim().trim_end_matches('%').trim();
    let normalized = normalize_digits(core)?;
    let mut value: f64 = normalized.parse().ok()?;
    if negative {
        value = -value;
    }
    Some(value)
}

fn normalize_digits(s: &str) -> Option<String> {
    let has_comma = s.contains(',');
    let has_dot = s.contains('.');
    if has_comma && has_dot {
        // Decide decimal separator by last occurrence.
        if s.rfind(',').unwrap_or(0) > s.rfind('.').unwrap_or(0) {
            // EU: 1.234,56
            Some(s.replace('.', "").replace(',', "."))
        } else {
            // US: 1,234.56
            Some(s.replace(',', ""))
        }
    } else if has_comma {
        let parts: Vec<_> = s.split(',').collect();
        if parts.len() > 1 && parts[1..].iter().all(|p| p.len() == 3) {
            // thousands: 14,000,000
            Some(s.replace(',', ""))
        } else if parts.len() == 2 {
            // decimal: 92,5
            Some(s.replace(',', "."))
        } else {
            Some(s.replace(',', ""))
        }
    } else if has_dot {
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() > 2 && parts[1..].iter().all(|p| p.len() == 3) {
            // EU thousands: 14.000.000
            Some(s.replace('.', ""))
        } else {
            Some(s.to_string())
        }
    } else {
        Some(s.to_string())
    }
}

fn multiplier_factor(word: &str) -> f64 {
    let w = word.trim().to_lowercase();
    if w.starts_with("млрд") || w.starts_with("миллиард") || w.starts_with("billion") {
        1_000_000_000.0
    } else if w.starts_with("млн") || w.starts_with("миллион") || w.starts_with("million")
    {
        1_000_000.0
    } else if w.starts_with("тыс") || w.starts_with("thousand") {
        1_000.0
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(text: &str) -> Vec<f64> {
        extract_numerics(text)
            .into_iter()
            .map(|n| n.value)
            .collect()
    }

    #[test]
    fn extracts_grouped_and_percent() {
        assert_eq!(
            values("Выручка составила 14,000,000 рублей"),
            vec![14_000_000.0]
        );
        assert_eq!(values("Ставка НДС 20%"), vec![20.0]);
        assert_eq!(values("Курс 92.5 ₽/$"), vec![92.5]);
        assert_eq!(values("±0.01"), vec![0.01]);
    }

    #[test]
    fn extracts_russian_million_shorthand() {
        assert_eq!(values("Выручка 14 млн"), vec![14_000_000.0]);
    }

    #[test]
    fn verify_numeric_tolerance() {
        let ctx = extract_numerics("context 14000000");
        assert!(verify_numeric(14_000_000.0, &ctx, DEFAULT_TOLERANCE));
        assert!(!verify_numeric(15_000_000.0, &ctx, DEFAULT_TOLERANCE));
    }

    #[test]
    fn verify_answer_against_context_ru() {
        assert!(verify_answer_against_context(
            "Выручка 14 млн",
            "В отчёте указано 14,000,000",
            DEFAULT_TOLERANCE
        ));
        assert!(!verify_answer_against_context(
            "Выручка 15 млн",
            "В отчёте указано 14,000,000",
            DEFAULT_TOLERANCE
        ));
    }

    #[test]
    fn no_numbers_passes() {
        assert!(verify_answer_against_context(
            "No digits here",
            "also none",
            DEFAULT_TOLERANCE
        ));
    }

    #[test]
    fn grounded_style_simple_floats() {
        // Parity with grounded-llm Go extractNumbersFromText for plain decimals.
        assert_eq!(values("Growth 748.5 cm and 31.8%"), vec![748.5, 31.8]);
    }

    #[test]
    fn unmatched_lists_hallucinations() {
        let bad = unmatched_numerics("Margin 72%", "No digits in text.", DEFAULT_TOLERANCE);
        assert_eq!(bad, vec![72.0]);
    }
}
