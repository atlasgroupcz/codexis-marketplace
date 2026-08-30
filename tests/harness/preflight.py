"""Bring the VM and the daemon inside it up, before a tier talks to them.

The order is load-bearing: the daemon runs inside this user's VM, so on a
stopped account it never reports UP and a health check alone would time out
saying the daemon is unreachable. That is true, and not the reason.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import Protocol

from tests.harness.output import Results


class DaemonHost(Protocol):
    """The two things preflight needs of a client.

    Narrower than DaemonClient on purpose: this is the whole dependency, so a
    test can stand in for it without standing up the rest of the client.
    """

    def ensure_vm_running(self, *, log: Callable[[str], None] | None = ...) -> None: ...

    def wait_healthy(self) -> bool: ...


def ensure_ready(client: DaemonHost, r: Results) -> None:
    """Block until the VM runs and the daemon in it reports UP.

    Raises RuntimeError if either step gives up, so a caller reports one failure
    the same way whichever step produced it.
    """
    r.log("Ensuring the VM is running...")
    client.ensure_vm_running(log=r.log)
    r.ok("VM is running")
    if not client.wait_healthy():
        raise RuntimeError("daemon not reachable or healthy within the startup window")
    r.ok("Daemon is healthy")
