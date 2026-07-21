"""Unit tests for scoping the skill listing to one marketplace plugin instance."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _skills import skills_of_plugin

OUR = "id-our-codexis"
OTHER = "id-onboarding-codexis"


def _skill(full_name, plugin_id):
    return {"fullName": full_name, "plugin": None if plugin_id is None else {"id": plugin_id}}


def test_includes_skills_from_the_given_plugin():
    skills = [_skill("codexis:codexis", OUR), _skill("codexis:cdxctl", OUR)]
    assert skills_of_plugin(skills, OUR) == {"codexis:codexis", "codexis:cdxctl"}


def test_excludes_the_same_skill_from_a_different_plugin_instance():
    """The onboarding preset installs the same plugin from another marketplace,
    so `codexis:codexis` is also provided under a different plugin id. Scoping by
    id must not count that copy as ours - this is the whole bug.
    """
    skills = [_skill("codexis:codexis", OTHER)]
    assert skills_of_plugin(skills, OUR) == set()


def test_ignores_skills_with_no_plugin():
    """LOCAL skills (and skills whose owning plugin is gone) report plugin=null."""
    skills = [_skill("local:thing", None)]
    assert skills_of_plugin(skills, OUR) == set()


def test_our_copy_survives_alongside_the_onboarding_copy():
    skills = [_skill("codexis:codexis", OUR), _skill("codexis:codexis", OTHER)]
    assert skills_of_plugin(skills, OUR) == {"codexis:codexis"}


if __name__ == "__main__":
    import pytest
    sys.exit(pytest.main([__file__]))
