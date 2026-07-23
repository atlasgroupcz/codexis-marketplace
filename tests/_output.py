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
    # When `record` is set, output is buffered into `lines` instead of printed.
    # Parallel plugin runs give each plugin its own recording Results, then
    # `merge` them back in manifest order so the log reads as one contiguous
    # block per plugin rather than interleaved across threads.
    record: bool = False
    lines: list[str] = field(default_factory=list)

    def _emit(self, text: str) -> None:
        if self.record:
            self.lines.append(text)
        else:
            print(text)

    def ok(self, msg: str) -> None:
        self.passed += 1
        self._emit(f"{C.G}  ✓{C.N} {msg}")

    def fail(self, msg: str) -> None:
        self.failed += 1
        self.errors.append(msg)
        self._emit(f"{C.R}  ✗{C.N} {msg}")

    def skip(self, msg: str) -> None:
        self.skipped += 1
        self._emit(f"{C.Y}  ⊘{C.N} {msg}")

    def section(self, msg: str) -> None:
        self._emit(f"\n{C.BOLD}━━━ {msg} ━━━{C.N}")

    def log(self, msg: str) -> None:
        self._emit(f"{C.B}[TEST]{C.N} {msg}")

    def merge(self, other: "Results") -> None:
        """Fold a recorded sub-result into this one: counts, errors, and its
        buffered lines replayed in order. Called single-threaded after the
        parallel workers finish, so nothing here needs its own lock."""
        self.passed += other.passed
        self.failed += other.failed
        self.skipped += other.skipped
        self.errors.extend(other.errors)
        for line in other.lines:
            self._emit(line)

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
