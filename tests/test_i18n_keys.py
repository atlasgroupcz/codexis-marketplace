"""A missing translation fails nothing at runtime - `t('a.b')` just renders the
key, so the UI shows `proceedings.filterUnconfirmed` where it meant "Nepotvrzené".
i18next reports it only as a `missingKey` line under `debug`, which is off in
tests (it buried the run in ~70 lines of config echo) and unread in CI anyway.

So the guard lives here instead, as a check that fails: every key an app asks
for must exist, and the three languages must stay in step.

Static by design: it reads `t('literal')` call sites, so a key assembled from a
variable is invisible to it. That is a smaller blind spot than it sounds - the
apps write literals today - and a partial guard beats the none we had.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
APPS_DIR = REPO / "frontend" / "apps"
LANGUAGES = ("cs", "en", "sk")
# `t('some.key')` / `t("some.key")`, ignoring anything interpolated.
T_CALL = re.compile(r"\bt\(\s*['\"]([a-zA-Z0-9_.]+)['\"]")


def _apps() -> list[str]:
    if not APPS_DIR.is_dir():
        return []
    return sorted(
        p.name for p in APPS_DIR.iterdir() if (p / "public" / "locales").is_dir()
    )


_APPS = _apps()


def _flatten(node: dict, prefix: str = "") -> set[str]:
    """Nested translation JSON to dotted keys, the form `t()` uses."""
    keys: set[str] = set()
    for key, value in node.items():
        if isinstance(value, dict):
            keys |= _flatten(value, f"{prefix}{key}.")
        else:
            keys.add(prefix + key)
    return keys


def _catalog(app: str, language: str) -> set[str]:
    path = APPS_DIR / app / "public" / "locales" / language / "translation.json"
    return (
        _flatten(json.loads(path.read_text(encoding="utf-8")))
        if path.is_file()
        else set()
    )


def _keys_used(app: str) -> set[str]:
    used: set[str] = set()
    for source in (APPS_DIR / app / "src").rglob("*.ts*"):
        used |= set(T_CALL.findall(source.read_text(encoding="utf-8")))
    return used


@pytest.mark.skipif(not _APPS, reason="no frontend apps with locales")
@pytest.mark.parametrize("app", _APPS)
def test_every_key_the_app_asks_for_is_translated(app: str) -> None:
    missing = sorted(_keys_used(app) - _catalog(app, "cs"))
    assert not missing, (
        f"{app}: {len(missing)} key(s) used in t() but absent from cs/translation.json "
        f"- the UI would render the key itself: {missing[:10]}"
    )


@pytest.mark.skipif(not _APPS, reason="no frontend apps with locales")
@pytest.mark.parametrize("app", _APPS)
def test_the_languages_stay_in_step(app: str) -> None:
    """A key present in one language and not another silently falls back to `en`
    (or to the raw key when `en` is the one missing it)."""
    catalogs = {language: _catalog(app, language) for language in LANGUAGES}
    present = {language: keys for language, keys in catalogs.items() if keys}
    union: set[str] = set().union(*present.values())

    gaps = {
        language: sorted(union - keys)[:10]
        for language, keys in present.items()
        if union - keys
    }
    assert not gaps, f"{app}: languages out of step: {gaps}"
