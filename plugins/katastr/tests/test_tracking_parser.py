"""Proceeding number parsing, formatting, normalization."""

import pytest

from katastr_core.exceptions import InvalidProceedingNumberError
from katastr_core.tracking import (
    format_proceeding_number,
    normalize_number,
    parse_proceeding_number,
)


class TestParseProceedingNumber:
    def test_valid_uppercase(self):
        result = parse_proceeding_number("V-123/2026-701")
        assert result == {
            "typ_rizeni": "V",
            "poradove_cislo": 123,
            "rok": 2026,
            "kod_pracoviste": 701,
        }

    def test_valid_lowercase_normalized_to_upper(self):
        result = parse_proceeding_number("v-123/2026-701")
        assert result["typ_rizeni"] == "V"

    def test_leading_trailing_whitespace_stripped(self):
        result = parse_proceeding_number("  V-123/2026-701  ")
        assert result["poradove_cislo"] == 123

    def test_multi_letter_type(self):
        result = parse_proceeding_number("OR-5/2025-801")
        assert result["typ_rizeni"] == "OR"
        assert result["poradove_cislo"] == 5

    def test_large_serial_number(self):
        result = parse_proceeding_number("V-9999999/2026-701")
        assert result["poradove_cislo"] == 9999999

    def test_empty_raises(self):
        with pytest.raises(InvalidProceedingNumberError):
            parse_proceeding_number("")

    def test_none_raises(self):
        with pytest.raises(InvalidProceedingNumberError):
            parse_proceeding_number(None)  # type: ignore[arg-type]

    @pytest.mark.parametrize("bad", [
        "V123/2026-701",        # missing dash after type
        "V-123-2026-701",       # dash instead of slash
        "V-123/26-701",         # 2-digit year
        "V-abc/2026-701",       # non-numeric serial
        "V-123/2026",           # missing workplace
        "1-123/2026-701",       # digit type
        "V-123/2026-XYZ",       # non-numeric workplace
        "V/123/2026/701",       # all slashes
        "random text",
    ])
    def test_invalid_format_raises(self, bad):
        with pytest.raises(InvalidProceedingNumberError):
            parse_proceeding_number(bad)


class TestFormatProceedingNumber:
    def test_roundtrip(self):
        parsed = parse_proceeding_number("V-123/2026-701")
        assert format_proceeding_number(parsed) == "V-123/2026-701"

    def test_canonical_from_lowercase(self):
        parsed = parse_proceeding_number("v-1/2026-701")
        assert format_proceeding_number(parsed) == "V-1/2026-701"


class TestNormalizeNumber:
    def test_identity_on_canonical(self):
        assert normalize_number("V-123/2026-701") == "V-123/2026-701"

    def test_uppercases_type(self):
        assert normalize_number("v-123/2026-701") == "V-123/2026-701"

    def test_strips_whitespace(self):
        assert normalize_number("  V-123/2026-701\n") == "V-123/2026-701"

    def test_invalid_propagates(self):
        with pytest.raises(InvalidProceedingNumberError):
            normalize_number("garbage")
