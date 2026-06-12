# ARES Plugin Structure

## Created Structure

```text
plugins/ares/
  .claude-plugin/plugin.json
  README.md
  STRUCTURE.md
  bin/ares
  hooks/install-binaries.sh
  hooks/uninstall-binaries.sh
  lib/ares_cli/__init__.py
  lib/ares_cli/__main__.py
  lib/ares_cli/cli.py
  lib/ares_cli/client.py
  lib/ares_cli/errors.py
  lib/ares_cli/formatters.py
  lib/ares_cli/models.py
  lib/ares_cli/service.py
  lib/ares_cli/sources.py
  skills/ares/SKILL.md
  skills/ares/references/cli.md
```

## File Contents

- `.claude-plugin/plugin.json` - plugin metadata, Czech/English/Slovak display text, install hooks and skill path.
- `README.md` - short project overview, final CLI command surface and output behavior.
- `STRUCTURE.md` - this file; documents structure and intended contents.
- `bin/ares` - executable Python wrapper that locates bundled `lib/ares_cli` and runs the CLI.
- `hooks/install-binaries.sh` - installs `ares` and copies the Python package to `~/.local/share/ares/lib`.
- `hooks/uninstall-binaries.sh` - removes installed executable and copied library.
- `lib/ares_cli/__init__.py` - package metadata and version.
- `lib/ares_cli/__main__.py` - enables `python -m ares_cli`.
- `lib/ares_cli/cli.py` - argparse command definitions and user-facing help.
- `lib/ares_cli/client.py` - HTTP boundary for ARES REST calls and API error handling.
- `lib/ares_cli/errors.py` - shared user-facing exception type.
- `lib/ares_cli/formatters.py` - normalization of raw ARES responses into legal-domain JSON.
- `lib/ares_cli/models.py` - small typed contracts for entity/source records.
- `lib/ares_cli/service.py` - command-level mapping: search, company, officers, trades, raw.
- `lib/ares_cli/sources.py` - known ARES source names and endpoint templates.
- `skills/ares/SKILL.md` - Czech instructions for Claude: when to use `ares`, which command to run, and how to answer.
- `skills/ares/references/cli.md` - compact command reference used by the skill.
