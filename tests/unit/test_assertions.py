from pathlib import Path
import re
import sys
import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from _assertions import substitute, matches_subset


# --- substitute ---------------------------------------------------

def test_substitute_replaces_single_var():
    assert substitute("id={{ x }}", {"x": "abc"}) == "id=abc"


def test_substitute_handles_no_vars():
    assert substitute("no vars here", {"x": "abc"}) == "no vars here"


def test_substitute_json_encodes_non_strings():
    assert substitute("v={{ x }}", {"x": 42}) == "v=42"


def test_substitute_raises_on_missing_var():
    with pytest.raises(KeyError, match="missing"):
        substitute("hi {{ missing }}", {})


# --- matches_subset ----------------------------------------------

def test_subset_literal_match_passes():
    assert matches_subset({"a": 1, "b": 2}, {"a": 1, "b": 2, "c": 3}) is None


def test_subset_missing_key_fails():
    err = matches_subset({"a": 1}, {"b": 1})
    assert err is not None and "missing key 'a'" in err


def test_subset_value_mismatch_fails():
    err = matches_subset({"a": 1}, {"a": 2})
    assert err is not None and "value mismatch" in err


def test_subset_regex_prefix_passes():
    assert matches_subset({"name": "~/^e2e-/"}, {"name": "e2e-foo"}) is None


def test_subset_regex_prefix_fails():
    err = matches_subset({"name": "~/^e2e-/"}, {"name": "prod-foo"})
    assert err is not None and "regex" in err


def test_subset_nested_dict_recurses():
    assert matches_subset({"a": {"b": 1}}, {"a": {"b": 1, "c": 2}}) is None
    err = matches_subset({"a": {"b": 1}}, {"a": {"b": 2}})
    assert err is not None
