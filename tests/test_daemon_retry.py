"""Unit tests for DaemonClient._open retry behaviour.

The marketplace test runs against a shared dev daemon that redeploys under long
runs, so a single request can hit a momentary gateway 405/5xx or a dropped
connection while the backend reloads. `_open` must ride those out instead of
failing the whole suite on one blip.
"""

from __future__ import annotations

import email.message
import sys
import urllib.error
import urllib.request
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import _daemon_client
from _daemon_client import _MAX_ATTEMPTS, _TRANSIENT_STATUS, DaemonClient


def _http_error(code: int) -> urllib.error.HTTPError:
    return urllib.error.HTTPError("http://x/graphql", code, "err",
                                  email.message.Message(), None)


@pytest.fixture(autouse=True)
def _no_sleep(monkeypatch):
    """Retries must not actually sleep during the unit tests."""
    monkeypatch.setattr(_daemon_client.time, "sleep", lambda *_a: None)


def _patch_urlopen(monkeypatch, responses):
    """`responses`: each item is either an exception (raised) or a value (returned)."""
    it = iter(responses)
    calls = []

    def fake(req, **_kwargs):
        calls.append(req)
        item = next(it)
        if isinstance(item, Exception):
            raise item
        return item

    monkeypatch.setattr(urllib.request, "urlopen", fake)
    return calls


def _client(refresher=None):
    return DaemonClient("http://x", "tok", token_refresher=refresher)


def _req():
    return urllib.request.Request("http://x/graphql")


def test_retries_a_transient_error_then_succeeds(monkeypatch):
    calls = _patch_urlopen(monkeypatch, [_http_error(503), _http_error(405), "OK"])
    assert _client()._open(_req()) == "OK"
    assert len(calls) == 3


def test_gives_up_after_max_attempts_on_a_persistent_transient(monkeypatch):
    calls = _patch_urlopen(monkeypatch, [_http_error(405)] * _MAX_ATTEMPTS)
    with pytest.raises(urllib.error.HTTPError):
        _client()._open(_req())
    assert len(calls) == _MAX_ATTEMPTS


def test_a_non_transient_error_is_not_retried(monkeypatch):
    calls = _patch_urlopen(monkeypatch, [_http_error(400), "unused"])
    with pytest.raises(urllib.error.HTTPError):
        _client()._open(_req())
    assert len(calls) == 1


def test_a_dropped_connection_is_retried(monkeypatch):
    calls = _patch_urlopen(monkeypatch, [urllib.error.URLError("reset"), "OK"])
    assert _client()._open(_req()) == "OK"
    assert len(calls) == 2


def test_401_refreshes_the_token_once_then_retries(monkeypatch):
    calls = _patch_urlopen(monkeypatch, [_http_error(401), "OK"])
    client = _client(refresher=lambda: "fresh")
    assert client._open(_req()) == "OK"
    assert len(calls) == 2
    assert client.auth_header == "Bearer fresh"


def test_401_without_a_working_refresher_is_not_retried(monkeypatch):
    calls = _patch_urlopen(monkeypatch, [_http_error(401), "unused"])
    with pytest.raises(urllib.error.HTTPError):
        _client()._open(_req())
    assert len(calls) == 1


def test_405_is_in_the_transient_set():
    # the exact code that killed build #390
    assert 405 in _TRANSIENT_STATUS


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))
