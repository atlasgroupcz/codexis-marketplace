"""Run the per-plugin install/uninstall checks, serially or across threads.

Each plugin's check is a short chain of blocking daemon round-trips (install,
list skills, uninstall, list skills). They are network-bound and independent -
the skill assertions are scoped to one plugin.id, so no plugin's result depends
on another's - which makes them safe to fan out across threads. Whether that
buys wall-clock depends on the daemon: if it serializes plugin operations per
user, the workers just queue on its side. So parallelism is opt-in (`workers`),
default 1 (unchanged serial behavior), and left for CI to measure and tune.

Regardless of worker count the output and counts are merged back in manifest
order, so a parallel run reads identically to a serial one.

Independent also means isolated when a check goes wrong: whatever one plugin
raises is reported against that plugin and the rest still run - see `_guarded`.
"""

from __future__ import annotations

from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor

from tests.harness.output import Results

# One plugin's check, given the plugin dict and the Results to report into.
RunOne = Callable[[dict, Results], None]


def _label(plugin: dict) -> str:
    """What to call a plugin in a failure line, whatever the manifest entry holds."""
    return plugin.get("name") or plugin.get("id") or "<unnamed plugin>"


def _guarded(run_one: RunOne, plugin: dict, r: Results) -> None:
    """Run one plugin's check, keeping whatever it raises inside that plugin.

    The suite's own handlers catch RuntimeError, which is what a GraphQL-level
    refusal arrives as. Everything else reaches here: a transient gateway 5xx is
    urllib's HTTPError, an OSError; a malformed expected.json is a
    JSONDecodeError. Unguarded, either unwinds past the loop and ends the
    process mid-suite, so the plugins after it never run and no summary is
    printed - which is how a daemon redeploy turned one plugin's blip into a
    bare traceback for the whole run.

    BaseException is deliberately not caught: ctrl-C still ends the run.
    """
    try:
        run_one(plugin, r)
    except Exception as exc:
        r.fail(f"{_label(plugin)}: {type(exc).__name__}: {exc}")


def run_plugins(
    plugins: list[dict],
    run_one: RunOne,
    r: Results,
    workers: int = 1,
) -> None:
    """Run `run_one` for every plugin, reporting into `r`.

    `workers <= 1` runs serially and reports straight into `r`. With more
    workers each plugin runs in its own thread against a fresh recording
    Results, then the sub-results are merged into `r` in manifest order
    (`pool.map` preserves input order), never as they finish.

    Either way every plugin is attempted: `_guarded` turns a check that raises
    into that plugin's failure line instead of the end of the run.
    """
    if workers <= 1 or len(plugins) <= 1:
        for plugin in plugins:
            _guarded(run_one, plugin, r)
        return

    def _work(plugin: dict) -> Results:
        local = Results(record=True)
        _guarded(run_one, plugin, local)
        return local

    with ThreadPoolExecutor(max_workers=workers) as pool:
        sub_results = list(pool.map(_work, plugins))

    for sub in sub_results:
        r.merge(sub)
