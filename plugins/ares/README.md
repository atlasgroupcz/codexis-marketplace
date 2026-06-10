# ARES Plugin

Python-first Claude plugin scaffold for querying the Czech ARES public register.

The plugin is intended for Czech-speaking legal and compliance users who need to verify Czech companies and entrepreneurs by name or IČO.

## CLI Surface

```bash
ares search <query>
ares company <ico>
ares officers <ico>
ares trades <ico>
ares owners <ico>
ares raw <ico> --source basic|vr|res|rzp|rpsh
```

The executable currently exposes the final command structure and clear help output. Network calls and response normalization are intentionally isolated in `lib/ares_cli/client.py`, `service.py` and `formatters.py` for the next implementation step.

## Key Files

- `bin/ares` - installed executable wrapper.
- `lib/ares_cli/` - Python package for CLI parsing, ARES calls and JSON formatting.
- `skills/ares/SKILL.md` - Czech Claude skill for legal users.
- `skills/ares/references/cli.md` - command reference for the skill.
- `hooks/` - install/uninstall scripts for the executable and bundled Python package.
- `STRUCTURE.md` - concise file-by-file structure documentation.
