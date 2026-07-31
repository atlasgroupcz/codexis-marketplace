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
"""

from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from typing import Callable

from tests._output import Results

# One plugin's check, given the plugin dict and the Results to report into.
RunOne = Callable[[dict, Results], None]


def run_plugins(
    plugins: list[dict],
    run_one: RunOne,
    r: Results,
    workers: int = 1,
) -> None:
    """Run `run_one` for every plugin, reporting into `r`.

    `workers <= 1` runs serially and reports straight into `r` - byte-for-byte
    the original behavior. With more workers each plugin runs in its own thread
    against a fresh recording Results, then the sub-results are merged into `r`
    in manifest order (`pool.map` preserves input order), never as they finish.
    """
    if workers <= 1 or len(plugins) <= 1:
        for plugin in plugins:
            run_one(plugin, r)
        return

    def _work(plugin: dict) -> Results:
        local = Results(record=True)
        run_one(plugin, local)
        return local

    with ThreadPoolExecutor(max_workers=workers) as pool:
        sub_results = list(pool.map(_work, plugins))

    for sub in sub_results:
        r.merge(sub)
