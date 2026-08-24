"""Unit tests for the shared daemon preflight.

Both daemon tiers carried their own copy of this sequence. What the copies
agreed on is what these tests pin: the VM comes up before the health check, and
either step giving up reaches the caller as one kind of failure.
"""

from __future__ import annotations

import pytest

from tests.harness.output import Results
from tests.harness.preflight import ensure_ready


class FakeClient:
    """Records the calls preflight makes, without a daemon behind them."""

    def __init__(self, *, vm_error: str | None = None, healthy: bool = True) -> None:
        self.calls: list[str] = []
        self._vm_error = vm_error
        self._healthy = healthy

    def ensure_vm_running(self, log=None) -> None:
        self.calls.append("ensure_vm_running")
        if self._vm_error is not None:
            raise RuntimeError(self._vm_error)

    def wait_healthy(self) -> bool:
        self.calls.append("wait_healthy")
        return self._healthy


def test_vm_comes_up_before_the_health_check():
    """The daemon runs inside the VM, so probing health first would time out
    reporting an unreachable daemon - true, and not the reason."""
    client = FakeClient()
    ensure_ready(client, Results(record=True))
    assert client.calls == ["ensure_vm_running", "wait_healthy"]


def test_a_daemon_that_never_reports_up_raises():
    with pytest.raises(RuntimeError, match="healthy"):
        ensure_ready(FakeClient(healthy=False), Results(record=True))


def test_a_vm_that_will_not_start_raises_without_probing_health():
    client = FakeClient(vm_error="VM entered terminal state STOPPED")
    with pytest.raises(RuntimeError, match="STOPPED"):
        ensure_ready(client, Results(record=True))
    assert "wait_healthy" not in client.calls


def test_both_milestones_are_reported():
    r = Results(record=True)
    ensure_ready(FakeClient(), r)
    assert r.passed == 2
