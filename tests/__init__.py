"""Package marker: the helpers here are imported as `tests.harness.daemon_client`, …

Without it they would be top-level modules whose directory has to be on
sys.path before the first import, and `tests/run.py` and
`tests/promptfoo/run.py` would compete for the single module name `run`.
"""
