"""Make sibling modules importable under pytest's importlib mode.

The repo's pyproject sets `--import-mode=importlib`, which (unlike the default
prepend mode) does NOT add a test file's directory to sys.path. The promptfoo
modules (`_arbitration`, `assertions`, `provider`, `setup`) import each other and
the shared `tests/` helpers (`_daemon_client`, `_chat_runner`) by bare name, so
both directories must be on the path for the unit tests to import them.
"""

import sys
from pathlib import Path

_HERE = Path(__file__).resolve().parent
for _p in (_HERE, _HERE.parent):  # tests/promptfoo and tests
    if str(_p) not in sys.path:
        sys.path.insert(0, str(_p))

# tests/promptfoo/run.py (the promptfoo runner) and tests/run.py (the
# top-level dispatcher, see Task 3) both sit on sys.path above and share the
# bare module name "run". Which one `import run` resolves to would otherwise
# depend on sys.path order at the moment of the *first* such import - and that
# order shifts as other test modules under tests/ (test_daemon_health.py,
# test_skills.py, ...) each prepend their own directory to sys.path when they
# get imported, so plain path ordering is not reliable across collection
# orders. Pin "run" in sys.modules to the promptfoo module explicitly, before
# any test in this directory can be collected: pytest always loads a
# directory's conftest.py before importing test files in it, so this always
# wins the race regardless of run order.
if "run" not in sys.modules:
    import importlib.util

    _spec = importlib.util.spec_from_file_location("run", _HERE / "run.py")
    assert _spec is not None and _spec.loader is not None
    _promptfoo_run = importlib.util.module_from_spec(_spec)
    sys.modules["run"] = _promptfoo_run
    _spec.loader.exec_module(_promptfoo_run)
