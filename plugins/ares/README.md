# ARES Plugin

Python-first Claude plugin for querying the Czech ARES public register.

The plugin is intended for Czech-speaking legal users who need to verify Czech companies and entrepreneurs by name or IČO.

## CLI Surface

```bash
ares search <query>
ares company <ico>
ares officers <ico>
ares trades <ico>
ares raw <ico> --source basic|vr|res|rzp
```

Mapped commands output JSON with an `echo` block and a normalized Czech legal summary. `ares raw` outputs the original ARES JSON without interpretation.

Output mappings are documented in `skills/ares/references/cli.md` and were checked against the attached ARES OpenAPI docs (`api-docs.json`) for the endpoint families used by this plugin.

## Key Files

- `bin/ares` - installed executable wrapper.
- `lib/ares_cli/` - Python package for CLI parsing, ARES calls and JSON formatting.
- `skills/ares/SKILL.md` - Czech Claude skill for legal users.
- `skills/ares/references/cli.md` - command reference for the skill.
- `hooks/` - install/uninstall scripts for the executable and bundled Python package.
- `STRUCTURE.md` - concise file-by-file structure documentation.
