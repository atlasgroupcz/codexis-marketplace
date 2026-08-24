"""Import a plugin's bin/ entry point as a module, for tests to poke at.

Those entry points ship without a .py extension - they are installed onto PATH
and run as commands - so a plain import cannot find them. This is only for them:
the tiers are importable modules and reach each other by name.
"""

from __future__ import annotations

import importlib.machinery
import importlib.util
from pathlib import Path
from types import ModuleType


def load_script(name: str, path: str | Path) -> ModuleType:
    """Load the script at `path` under the module name `name`.

    SourceFileLoader is passed explicitly because the file has no extension for
    importlib to recognise it by.

    Both `spec` and `spec.loader` are typed optional: they are None when no
    loader claims the path, which cannot happen once one is handed over. The
    assertion states that once, instead of twelve call sites each dereferencing
    an Optional and a type checker objecting twelve times.
    """
    loader = importlib.machinery.SourceFileLoader(name, str(path))
    spec = importlib.util.spec_from_file_location(name, path, loader=loader)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
