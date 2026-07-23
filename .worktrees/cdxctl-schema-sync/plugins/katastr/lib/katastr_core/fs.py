"""Filesystem utilities shared between settings (API key) and tracking (state)."""

import json
import os
import tempfile


def atomic_replace(path, write_fn, mode=None):
    """Atomically replace `path`: tmp file + rename.

    `write_fn` receives an open text-mode file handle and writes the body.
    `mode` — if given, the tmp file is chmod'ed before the rename (use 0o600
    for files containing secrets).
    """
    d = os.path.dirname(path) or "."
    os.makedirs(d, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=os.path.basename(path) + ".", dir=d)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            write_fn(f)
        if mode is not None:
            os.chmod(tmp, mode)
        os.replace(tmp, path)
    except Exception:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def atomic_write_json(path, data):
    """Atomically write `data` as pretty-printed JSON to `path`."""
    def _write(f):
        json.dump(data, f, indent=2, ensure_ascii=False)
        f.write("\n")
    atomic_replace(path, _write)
