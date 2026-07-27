package rules

import (
	"math"
	"regexp"
	"strconv"
	"strings"
	"unicode"
)

// DefaultTolerance matches Grounded LLM / Rust grounded-guardrails (±0.01 absolute).
const DefaultTolerance = 0.01

var (
	reUSGrouped = regexp.MustCompile(`[±+\-]?\d{1,3}(?:,\d{3})+(?:\.\d+)?(?:\s*%)?`)
	reEUGrouped = regexp.MustCompile(`[±+\-]?\d{1,3}(?:\.\d{3})+(?:,\d+)?(?:\s*%)?`)
	rePlain     = regexp.MustCompile(`[±+\-]?\d+(?:[.,]\d+)?(?:\s*%)?`)
)

// NumericValue is a parsed number span.
type NumericValue struct {
	Value float64
	Start int
	End   int
	Raw   string
}

// ExtractNumerics finds numbers in text (grouped digits, %, RU/EN multipliers).
func ExtractNumerics(text string) []NumericValue {
	type span struct {
		start, end int
		raw        string
	}
	var spans []span
	seen := map[[2]int]bool{}
	addMatches := func(re *regexp.Regexp) {
		for _, loc := range re.FindAllStringIndex(text, -1) {
			key := [2]int{loc[0], loc[1]}
			if seen[key] {
				continue
			}
			// Prefer longer matches: skip if fully inside an existing span.
			inside := false
			for _, s := range spans {
				if loc[0] >= s.start && loc[1] <= s.end {
					inside = true
					break
				}
			}
			if inside {
				continue
			}
			seen[key] = true
			spans = append(spans, span{start: loc[0], end: loc[1], raw: text[loc[0]:loc[1]]})
		}
	}
	addMatches(reUSGrouped)
	addMatches(reEUGrouped)
	addMatches(rePlain)

	out := make([]NumericValue, 0, len(spans))
	for _, s := range spans {
		v, ok := parseLexeme(s.raw)
		if !ok {
			continue
		}
		end := s.end
		if factor, consumed := readMultiplier(text[s.end:]); consumed > 0 {
			v *= factor
			end = s.end + consumed
		}
		out = append(out, NumericValue{
			Value: v,
			Start: s.start,
			End:   end,
			Raw:   text[s.start:end],
		})
	}
	return out
}

// VerifyNumeric reports whether value appears in context within tolerance.
func VerifyNumeric(value float64, context []NumericValue, tolerance float64) bool {
	for _, c := range context {
		if math.Abs(c.Value-value) <= tolerance {
			return true
		}
	}
	return false
}

// VerifyAnswerAgainstContext mirrors Rust / Grounded LLM semantics.
func VerifyAnswerAgainstContext(answer, contextText string, tolerance float64) bool {
	ans := ExtractNumerics(answer)
	if len(ans) == 0 {
		return true
	}
	ctx := ExtractNumerics(contextText)
	for _, n := range ans {
		if !VerifyNumeric(n.Value, ctx, tolerance) {
			return false
		}
	}
	return true
}

// UnmatchedNumerics returns answer numbers missing from context.
func UnmatchedNumerics(answer, contextText string, tolerance float64) []float64 {
	ctx := ExtractNumerics(contextText)
	var missing []float64
	for _, n := range ExtractNumerics(answer) {
		if !VerifyNumeric(n.Value, ctx, tolerance) {
			missing = append(missing, n.Value)
		}
	}
	return missing
}

func parseLexeme(raw string) (float64, bool) {
	trimmed := strings.TrimSpace(raw)
	negative := false
	body := trimmed
	switch {
	case strings.HasPrefix(body, "±"):
		body = strings.TrimPrefix(body, "±")
	case strings.HasPrefix(body, "+"):
		body = strings.TrimPrefix(body, "+")
	case strings.HasPrefix(body, "-"):
		negative = true
		body = strings.TrimPrefix(body, "-")
	}
	body = strings.TrimSpace(strings.TrimSuffix(strings.TrimSpace(body), "%"))
	norm, ok := normalizeDigits(body)
	if !ok {
		return 0, false
	}
	v, err := strconv.ParseFloat(norm, 64)
	if err != nil {
		return 0, false
	}
	if negative {
		v = -v
	}
	return v, true
}

func normalizeDigits(s string) (string, bool) {
	hasComma := strings.Contains(s, ",")
	hasDot := strings.Contains(s, ".")
	switch {
	case hasComma && hasDot:
		if strings.LastIndex(s, ",") > strings.LastIndex(s, ".") {
			return strings.ReplaceAll(strings.ReplaceAll(s, ".", ""), ",", "."), true
		}
		return strings.ReplaceAll(s, ",", ""), true
	case hasComma:
		parts := strings.Split(s, ",")
		if len(parts) > 1 {
			allThousands := true
			for _, p := range parts[1:] {
				if len(p) != 3 || !isAllDigits(p) {
					allThousands = false
					break
				}
			}
			if allThousands {
				return strings.ReplaceAll(s, ",", ""), true
			}
			if len(parts) == 2 {
				return strings.ReplaceAll(s, ",", "."), true
			}
		}
		return strings.ReplaceAll(s, ",", ""), true
	case hasDot:
		parts := strings.Split(s, ".")
		if len(parts) > 2 {
			allThousands := true
			for _, p := range parts[1:] {
				if len(p) != 3 || !isAllDigits(p) {
					allThousands = false
					break
				}
			}
			if allThousands {
				return strings.ReplaceAll(s, ".", ""), true
			}
		}
		return s, true
	default:
		return s, true
	}
}

func isAllDigits(s string) bool {
	for _, r := range s {
		if !unicode.IsDigit(r) {
			return false
		}
	}
	return len(s) > 0
}

func readMultiplier(rest string) (factor float64, consumed int) {
	trimmed := strings.TrimLeftFunc(rest, unicode.IsSpace)
	lead := len(rest) - len(trimmed)
	lower := strings.ToLower(trimmed)
	type cand struct {
		prefix string
		factor float64
	}
	candidates := []cand{
		{"billion", 1_000_000_000},
		{"million", 1_000_000},
		{"thousand", 1_000},
		{"миллиардов", 1_000_000_000},
		{"миллиарда", 1_000_000_000},
		{"миллиард", 1_000_000_000},
		{"млрд", 1_000_000_000},
		{"миллионов", 1_000_000},
		{"миллиона", 1_000_000},
		{"миллион", 1_000_000},
		{"млн", 1_000_000},
		{"тысячи", 1_000},
		{"тысяча", 1_000},
		{"тыс.", 1_000},
		{"тыс", 1_000},
	}
	for _, c := range candidates {
		if !strings.HasPrefix(lower, c.prefix) {
			continue
		}
		prefixRunes := len([]rune(c.prefix))
		orig := []rune(trimmed)
		if len(orig) < prefixRunes {
			continue
		}
		return c.factor, lead + len(string(orig[:prefixRunes]))
	}
	return 1, 0
}
