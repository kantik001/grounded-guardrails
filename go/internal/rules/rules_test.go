package rules

import "testing"

func TestExtractGroupedAndPercent(t *testing.T) {
	got := ExtractNumerics("Выручка составила 14,000,000 рублей")
	if len(got) != 1 || got[0].Value != 14_000_000 {
		t.Fatalf("got %#v", got)
	}
	got = ExtractNumerics("Ставка НДС 20%")
	if len(got) != 1 || got[0].Value != 20 {
		t.Fatalf("got %#v", got)
	}
	got = ExtractNumerics("±0.01")
	if len(got) != 1 || got[0].Value != 0.01 {
		t.Fatalf("got %#v", got)
	}
}

func TestRussianMillion(t *testing.T) {
	got := ExtractNumerics("Выручка 14 млн")
	if len(got) != 1 || got[0].Value != 14_000_000 {
		t.Fatalf("got %#v", got)
	}
}

func TestVerifyAnswer(t *testing.T) {
	if !VerifyAnswerAgainstContext("Выручка 14 млн", "14,000,000", DefaultTolerance) {
		t.Fatal("expected pass")
	}
	if VerifyAnswerAgainstContext("Выручка 15 млн", "14,000,000", DefaultTolerance) {
		t.Fatal("expected fail")
	}
}

func TestPII(t *testing.T) {
	if !ContainsPII("user@example.com") {
		t.Fatal("email")
	}
	if !ContainsPII("+1 (555) 123-4567") {
		t.Fatal("phone")
	}
	if ContainsPII("version 1.2.3") || ContainsPII("order #123-456") {
		t.Fatal("false positive")
	}
	ms := DetectPII("SSN 123-45-6789")
	if len(ms) != 1 || ms[0].Masked != "***-**-6789" {
		t.Fatalf("%#v", ms)
	}
}
