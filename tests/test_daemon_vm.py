"""Unit tests for the VM lifecycle on DaemonClient.

Installing a plugin writes into the account's VM, so a suite pointed at a
stopped account has nowhere to install to. These cover the start retry, the
readiness wait, and the two ways it gives up.
"""

from __future__ import annotations

import sys

import pytest

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


def _client(monkeypatch, starts=None, states=None):
    """A client whose start_vm/vm_status replay the given sequences.

    An entry that is an Exception instance is raised instead of returned, which
    is how a transient gateway failure reaches the code under test.
    """
    client = DaemonClient("http://x", "tok")

    def _replay(name, sequence):
        it = iter(sequence)

        def _call():
            value = next(it)
            if isinstance(value, Exception):
                raise value
            return value

        monkeypatch.setattr(client, name, _call)

    if starts is not None:
        _replay("start_vm", starts)
    if states is not None:
        _replay("vm_status", states)
    return client


VM_OK = {"__typename": "VirtualMachine", "status": "PROVISIONING"}


def _busy(message="temporarily unavailable"):
    return {"__typename": "ActionError", "message": message}


# --------------------------------------------------------------- start_vm
def test_start_vm_returns_the_union_member(monkeypatch):
    client = DaemonClient("http://x", "tok")
    monkeypatch.setattr(
        client, "gql_data", lambda *a, **k: {"startVirtualMachine": VM_OK}
    )
    assert client.start_vm() == VM_OK


def test_start_vm_survives_a_null_payload(monkeypatch):
    client = DaemonClient("http://x", "tok")
    monkeypatch.setattr(
        client, "gql_data", lambda *a, **k: {"startVirtualMachine": None}
    )
    assert client.start_vm() == {}


def test_start_vm_asks_the_gateway_not_the_daemon(monkeypatch):
    client = DaemonClient("http://x", "tok")
    seen: dict = {}

    def _gql_data(query, variables=None, url=None):
        seen["url"] = url
        return {"startVirtualMachine": VM_OK}

    monkeypatch.setattr(client, "gql_data", _gql_data)
    client.start_vm()
    assert seen["url"] == "http://x/gateway/graphql"


# -------------------------------------------------------------- vm_status
def test_vm_status_reads_the_status(monkeypatch):
    client = DaemonClient("http://x", "tok")
    monkeypatch.setattr(
        client,
        "gql_data",
        lambda *a, **k: {"me": {"virtualMachine": {"status": "RUNNING"}}},
    )
    assert client.vm_status() == "RUNNING"


@pytest.mark.parametrize(
    "payload",
    [
        {"me": None},
        {"me": {"virtualMachine": None}},
        {"me": {"virtualMachine": {"status": None}}},
    ],
)
def test_vm_status_is_unknown_when_the_gateway_reports_none(monkeypatch, payload):
    client = DaemonClient("http://x", "tok")
    monkeypatch.setattr(client, "gql_data", lambda *a, **k: payload)
    assert client.vm_status() == "UNKNOWN"


# ------------------------------------------------------- ensure_vm_running
def test_returns_as_soon_as_the_vm_is_running(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, starts=[VM_OK], states=["RUNNING"])
    client.ensure_vm_running(sleep=clock.sleep, now=clock.now)
    assert clock.sleeps == []  # never waited


def test_waits_through_provisioning(monkeypatch):
    clock = FakeClock()
    client = _client(
        monkeypatch, starts=[VM_OK], states=["PROVISIONING", "PROVISIONING", "RUNNING"]
    )
    client.ensure_vm_running(interval_s=5, sleep=clock.sleep, now=clock.now)
    assert clock.sleeps == [5, 5]


def test_retries_the_start_with_backoff_while_the_service_is_busy(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, starts=[_busy(), _busy(), VM_OK], states=["RUNNING"])
    client.ensure_vm_running(sleep=clock.sleep, now=clock.now)
    assert clock.sleeps == [1, 2]  # exponential, and no poll wait after


def test_a_raising_start_counts_as_a_failed_attempt(monkeypatch):
    clock = FakeClock()
    client = _client(
        monkeypatch, starts=[RuntimeError("502"), VM_OK], states=["RUNNING"]
    )
    client.ensure_vm_running(sleep=clock.sleep, now=clock.now)
    assert clock.sleeps == [1]


def test_gives_up_when_the_start_is_never_accepted(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, starts=[_busy("nope")] * 3)
    with pytest.raises(RuntimeError, match="did not succeed after 3 attempts"):
        client.ensure_vm_running(start_attempts=3, sleep=clock.sleep, now=clock.now)


def test_reports_the_gateway_message_when_it_gives_up(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, starts=[_busy("out of capacity")] * 2)
    with pytest.raises(RuntimeError, match="out of capacity"):
        client.ensure_vm_running(start_attempts=2, sleep=clock.sleep, now=clock.now)


def test_tolerates_stopped_while_the_start_settles(monkeypatch):
    clock = FakeClock()
    client = _client(
        monkeypatch, starts=[VM_OK], states=["STOPPED", "PROVISIONING", "RUNNING"]
    )
    client.ensure_vm_running(interval_s=5, sleep=clock.sleep, now=clock.now)
    assert clock.sleeps == [5, 5]


def test_aborts_when_stopped_outlasts_the_settle_window(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, starts=[VM_OK], states=["STOPPED"] * 100)
    with pytest.raises(RuntimeError, match="terminal state STOPPED"):
        client.ensure_vm_running(
            settle_s=10, interval_s=5, sleep=clock.sleep, now=clock.now
        )
    assert clock.sleeps == [5, 5]


def test_aborts_on_a_terminal_stopped(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, starts=[VM_OK], states=["PROVISIONING", "STOPPED"])
    with pytest.raises(RuntimeError, match="terminal state STOPPED"):
        client.ensure_vm_running(sleep=clock.sleep, now=clock.now)


def test_keeps_polling_through_a_transient_status_failure(monkeypatch):
    clock = FakeClock()
    client = _client(
        monkeypatch, starts=[VM_OK], states=[RuntimeError("503"), "RUNNING"]
    )
    client.ensure_vm_running(sleep=clock.sleep, now=clock.now)
    assert clock.sleeps == [5]


def test_gives_up_when_the_vm_never_comes_up(monkeypatch):
    clock = FakeClock()
    client = _client(monkeypatch, starts=[VM_OK], states=["PROVISIONING"] * 100)
    with pytest.raises(RuntimeError, match="did not reach RUNNING within 12s"):
        client.ensure_vm_running(
            timeout_s=12, interval_s=5, sleep=clock.sleep, now=clock.now
        )


def test_reports_progress_through_the_log_hook(monkeypatch):
    clock = FakeClock()
    client = _client(
        monkeypatch, starts=[_busy(), VM_OK], states=["PROVISIONING", "RUNNING"]
    )
    lines: list[str] = []
    client.ensure_vm_running(sleep=clock.sleep, now=clock.now, log=lines.append)
    assert any("retrying" in line for line in lines)
    assert any("PROVISIONING" in line for line in lines)
    assert any("RUNNING" in line for line in lines)


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))
