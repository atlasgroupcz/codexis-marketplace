"""ANSI colors and a pass/fail/skip result tracker for test scripts."""

from __future__ import annotations

from dataclasses import dataclass, field


class C:
    R = "\033[0;31m"
    G = "\033[0;32m"
    Y = "\033[1;33m"
    B = "\033[0;36m"
    BOLD = "\033[1m"
    N = "\033[0m"


@dataclass
class Results:
    passed: int = 0
    failed: int = 0
    skipped: int = 0
    errors: list[str] = field(default_factory=list)

    def ok(self, msg: str) -> None:
        self.passed += 1
        print(f"{C.G}  ✓{C.N} {msg}")

    def fail(self, msg: str) -> None:
        self.failed += 1
        self.errors.append(msg)
        print(f"{C.R}  ✗{C.N} {msg}")

    def skip(self, msg: str) -> None:
        self.skipped += 1
        print(f"{C.Y}  ⊘{C.N} {msg}")

    def section(self, msg: str) -> None:
        print(f"\n{C.BOLD}━━━ {msg} ━━━{C.N}")

    def log(self, msg: str) -> None:
        print(f"{C.B}[TEST]{C.N} {msg}")

    def summary(self) -> int:
        self.section("Results")
        print(f"  {C.G}Passed:  {self.passed}{C.N}")
        print(f"  {C.R}Failed:  {self.failed}{C.N}")
        if self.skipped:
            print(f"  {C.Y}Skipped: {self.skipped}{C.N}")
        print()
        if self.errors:
            print(f"{C.R}{C.BOLD}FAILURES:{C.N}")
            for e in self.errors:
                print(f"  {C.R}✗{C.N} {e}")
            print()
            return 1
        print(f"{C.G}{C.BOLD}All checks passed!{C.N}")
        return 0
