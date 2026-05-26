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
