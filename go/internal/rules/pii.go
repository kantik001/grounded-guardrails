package rules

import (
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

var (
	reEmail = regexp.MustCompile(`(?i)\b[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}\b`)
	rePhone = regexp.MustCompile(`(?:\+\s*1[\s.\-]*\(\s*\d{3}\s*\)[\s.\-]*\d{3}[\s.\-]*\d{4}|\+\s*7[\s.\-]*\d{3}[\s.\-]*\d{3}[\s.\-]*\d{2}[\s.\-]*\d{2}|\b\d{3}[.\-]\d{3}[.\-]\d{4}\b|\b\(\d{3}\)\s*\d{3}[\s.\-]*\d{4}\b)`)
	reSSN   = regexp.MustCompile(`\b\d{3}-\d{2}-\d{4}\b`)
	reCard  = regexp.MustCompile(`\b(?:\d{4}[\s\-]\d{4}[\s\-]\d{4}[\s\-]\d{4}|\d{16})\b`)
)

// PiiType classifies a PII span.
type PiiType string

const (
	PiiEmail       PiiType = "email"
	PiiPhone       PiiType = "phone"
	PiiSSN         PiiType = "ssn"
	PiiCreditCard  PiiType = "credit_card"
)

// PiiMatch is one detected span.
type PiiMatch struct {
	Type   PiiType
	Start  int
	End    int
	Masked string
}

// ContainsPII reports whether any PII pattern matches.
func ContainsPII(text string) bool {
	return reEmail.MatchString(text) ||
		rePhone.MatchString(text) ||
		reSSN.MatchString(text) ||
		reCard.MatchString(text)
}

// DetectPII returns all PII spans with masks.
func DetectPII(text string) []PiiMatch {
	var out []PiiMatch
	collect := func(re *regexp.Regexp, t PiiType, mask func(string) string) {
		for _, loc := range re.FindAllStringIndex(text, -1) {
			raw := text[loc[0]:loc[1]]
			out = append(out, PiiMatch{
				Type:   t,
				Start:  loc[0],
				End:    loc[1],
				Masked: mask(raw),
			})
		}
	}
	collect(reEmail, PiiEmail, maskEmail)
	collect(rePhone, PiiPhone, maskPhone)
	collect(reSSN, PiiSSN, maskSSN)
	collect(reCard, PiiCreditCard, maskCard)
	return out
}

func maskEmail(raw string) string {
	at := strings.IndexByte(raw, '@')
	if at <= 0 {
		return "***"
	}
	r := []rune(raw[:at])
	first := "*"
	if len(r) > 0 {
		first = string(r[0])
	}
	return first + "***@" + raw[at+1:]
}

func maskPhone(raw string) string {
	digits := digitsOnly(raw)
	if len(digits) < 4 {
		return "***"
	}
	return "***-***-" + digits[len(digits)-4:]
}

func maskSSN(raw string) string {
	digits := digitsOnly(raw)
	if len(digits) >= 4 {
		return "***-**-" + digits[len(digits)-4:]
	}
	return "***-**-****"
}

func maskCard(raw string) string {
	digits := digitsOnly(raw)
	if len(digits) >= 4 {
		return "**** **** **** " + digits[len(digits)-4:]
	}
	return "**** **** **** ****"
}

func digitsOnly(s string) string {
	var b strings.Builder
	for _, r := range s {
		if unicode.IsDigit(r) {
			b.WriteRune(r)
		}
	}
	return b.String()
}

// FormatPIIViolation builds a stable violation string for API responses.
func FormatPIIViolation(m PiiMatch) string {
	return fmt.Sprintf("pii:%s:%s", m.Type, m.Masked)
}
