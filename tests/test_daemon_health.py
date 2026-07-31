"""Unit tests for DaemonClient.wait_healthy.

The shared dev daemon redeploys under long runs and can be briefly down exactly
when the suite starts, so preflight must wait for it to come up instead of
aborting on the first probe.
"""

from __future__ import annotations

import sys
from pathlib import Path

from tests._daemon_client import DaemonClient


class FakeClock:
    def __init__(self):
        self.t = 0.0
        self.sleeps: list[float] = []

    def now(self):
        return self.t

    def sleep(self, seconds):
        self.sleeps.append(seconds)
        self.t += seconds


def _client(monkeypatch, health_states):
    client = DaemonClient("http://x", "tok")
    it = iter(health_states)
    monkeypatch.setattr(client, "health_check", lambda: next(it))
    return client


def test_returns_true_immediately_when_already_healthy(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, [True])
    assert client.wait_healthy(sleep=clock.sleep, now=clock.now) is True
    assert clock.sleeps == []  # never waited


def test_waits_until_the_daemon_comes_up(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, [False, False, True])
    assert client.wait_healthy(timeout_s=100, interval_s=3,
                               sleep=clock.sleep, now=clock.now) is True
    assert clock.sleeps == [3, 3]


def test_returns_false_when_it_never_comes_up(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, [False] * 100)
    assert client.wait_healthy(timeout_s=6, interval_s=3,
                               sleep=clock.sleep, now=clock.now) is False


if __name__ == "__main__":
    import pytest
    sys.exit(pytest.main([__file__]))
