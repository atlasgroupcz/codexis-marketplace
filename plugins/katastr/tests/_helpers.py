"""Shared test-data builders + CLI-module loader for katastr unit tests."""

import importlib.machinery
import importlib.util
import os


def _cuzk_record(stav="V řízení", stav_uhrady="N", operations=None, record_id="api-id-xyz"):
    return {
        "id": record_id,
        "stav": stav,
        "stavUhrady": stav_uhrady,
        "datumPrijeti": "2026-04-01",
        "provedeneOperace": operations or [],
    }


def _load_cli():
    """Load bin/katastr-cli as an in-process module (defs only; main() is
    guarded by __main__), so its cmd_* functions can be called and its
    module-level imports (tracking, notify, notify_settings, ...) monkeypatched
    directly. Used by CLI-wiring tests that need to observe/mock internals —
    something a subprocess-based invocation can't do."""
    path = os.path.join(os.path.dirname(__file__), "..", "bin", "katastr-cli")
    # bin/katastr-cli has no .py suffix, so spec_from_file_location can't
    # auto-detect a loader from the extension — pass one explicitly.
    loader = importlib.machinery.SourceFileLoader("katastr_cli", path)
    spec = importlib.util.spec_from_file_location("katastr_cli", path, loader=loader)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _load_component_binary():
    """Load components/katastr/katastr.py (the CGI binary) as an in-process
    module (defs only; main() is guarded by __main__), so handle_get/handle_post
    can be called directly and its module-level imports (tracking, notify,
    notify_settings, ...) monkeypatched — mirrors _load_cli() above."""
    path = os.path.join(
        os.path.dirname(__file__), "..", "components", "katastr", "katastr.py"
    )
    spec = importlib.util.spec_from_file_location("katastr_component", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
