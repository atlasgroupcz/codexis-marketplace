"""Subject input validation: IČO checksum/padding, date of birth, names."""

import pytest
from insolvence_core.exceptions import InvalidSubjectError
from insolvence_core.tracking import (
    normalize_birthdate,
    normalize_ico,
    normalize_jmeno,
    normalize_prijmeni,
)


class TestNormalizeIco:
    def test_valid(self):
        assert normalize_ico("04863313") == "04863313"
        assert normalize_ico("00006947") == "00006947"

    def test_strips_non_digits(self):
        assert normalize_ico("CZ 048 633 13") == "04863313"

    def test_left_pads_short_input(self):
        # 6947 is a valid IČO once zero-padded to 00006947.
        assert normalize_ico("6947") == "00006947"

    def test_rejects_bad_checksum(self):
        with pytest.raises(InvalidSubjectError):
            normalize_ico("12345678")

    def test_rejects_empty(self):
        with pytest.raises(InvalidSubjectError):
            normalize_ico("")

    def test_rejects_too_long(self):
        with pytest.raises(InvalidSubjectError):
            normalize_ico("123456789")


class TestNormalizeBirthdate:
    def test_iso(self):
        assert normalize_birthdate("1980-05-01") == "1980-05-01"

    def test_czech_dotted(self):
        assert normalize_birthdate("1.5.1980") == "1980-05-01"
        assert normalize_birthdate("01. 05. 1980") == "1980-05-01"

    def test_rejects_empty(self):
        with pytest.raises(InvalidSubjectError):
            normalize_birthdate("")

    def test_rejects_garbage(self):
        with pytest.raises(InvalidSubjectError):
            normalize_birthdate("not-a-date")


class TestNormalizeNames:
    def test_prijmeni_collapses_whitespace(self):
        assert normalize_prijmeni("  Novák  ") == "Novák"

    def test_prijmeni_required(self):
        with pytest.raises(InvalidSubjectError):
            normalize_prijmeni("")
        with pytest.raises(InvalidSubjectError):
            normalize_prijmeni("X")

    def test_jmeno_optional(self):
        assert normalize_jmeno("") == ""
        assert normalize_jmeno("  Jan  ") == "Jan"
