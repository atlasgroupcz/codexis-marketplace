# tests/conftest.py
# Registers CLI options needed by test-plugin-e2e.py.
# pytest_addoption must live in conftest.py (not in a test module) so that
# options are known at argument-parse time, before test collection begins.


def pytest_addoption(parser):
    g = parser.getgroup("plugin-e2e")
    g.addoption("--daemon", action="store", help="cdx-daemon base URL")
    g.addoption("--token", action="store", help="JWT bearer token")
    g.addoption("--git-url", action="store", help="Marketplace git URL")
    g.addoption("--git-ref", action="store", help="Marketplace git ref (merge-result)")
    g.addoption("--base-ref", action="store", default="origin/main",
                help="Base ref for change detection (default: origin/main)")
    g.addoption("--changed", action="store_true",
                help="Run only plugins changed between --base-ref and HEAD")
    g.addoption("--only", action="store", default="",
                help="Comma-separated list of plugin names; overrides --changed")
    g.addoption("--transcript-dir", action="store",
                default="test-results/transcripts",
                help="Where to write failure transcripts")
