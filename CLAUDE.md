# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **Claude Code Plugin Marketplace** maintained by ATLAS GROUP. It contains plugins that extend Claude Code with legal database access and data visualization capabilities.

## Repository Architecture

```
codexis-marketplace/
├── .claude-plugin/
│   └── marketplace.json          # Marketplace definition - lists all plugins
└── plugins/
    ├── codexis/                  # CODEXIS legal database API plugin
    │   ├── SKILL.md              # Main skill definition
    │   └── references/           # Detailed API documentation (12 files)
    └── visualization/            # D3.js visualization plugin
        ├── .claude-plugin/
        │   └── plugin.json       # Plugin manifest
        ├── skills/visualize/
        │   ├── SKILL.md          # Router skill
        │   └── references/       # 10 visualization types (chart, graph, grid, etc.)
        └── shared/lib/           # Shared JS libraries (D3, Tailwind, etc.)
```

## Adding a New Plugin to the Marketplace

Edit `.claude-plugin/marketplace.json` and add an entry to the `plugins` array:

```json
{
  "name": "plugin-name",
  "description": "Brief description of what the plugin does",
  "source": "./plugins/plugin-name",
  "category": "category-name"
}
```

## Plugin Structure

### Skill-Only Plugin (like codexis)

Minimal structure with just a SKILL.md:

```
plugins/plugin-name/
├── SKILL.md              # Main skill with YAML frontmatter
└── references/           # Optional: detailed documentation files
```

### Full Plugin (like visualization)

Includes manifest and shared resources:

```
plugins/plugin-name/
├── .claude-plugin/
│   └── plugin.json       # Plugin manifest (name, version, author, skills path)
├── skills/
│   └── skill-name/
│       ├── SKILL.md      # Skill definition
│       └── references/   # Type-specific implementations
└── shared/               # Shared resources
```

## Skill Definition Format

Skills are markdown files with YAML frontmatter:

```markdown
---
name: skill-name
description: When this skill should be triggered - include trigger keywords
version: 1.0.0
---

# Skill Title

Documentation and instructions...
```

## Plugin Manifest Format (plugin.json)

```json
{
  "name": "plugin-name",
  "version": "1.0.0",
  "description": "Plugin description",
  "author": {
    "name": "Author Name",
    "email": "email@example.com"
  },
  "keywords": ["relevant", "keywords"],
  "license": "PROPRIETARY",
  "postInstall": "command to run after plugin installation (e.g. copy binaries from bin/ to system PATH)",
  "postUninstall": "command to run after plugin uninstallation (e.g. remove binaries from system PATH)",
  "onUpdate": "command to run after plugin update (e.g. replace binaries in system PATH)",
  "skills": "./skills"
}
```

### Lifecycle Hooks

Plugins support three lifecycle hooks executed as shell commands by the harness:

- **`postInstall`** — runs after the plugin is installed
- **`postUninstall`** — runs after the plugin is uninstalled
- **`onUpdate`** — runs after the plugin is updated

The variable `${PLUGIN_DIR}` is available in hook commands and resolves to the plugin's installation directory.

### Executables / Binaries

Plugin executables must be placed in a `bin/` folder inside the plugin directory. Lifecycle hooks are typically used to install these binaries into the user-local `${HOME}/.local/bin/` location on install/update and remove them on uninstall. The cdx guest VM has `${HOME}/.local/bin` on `PATH` (both for interactive login shells and for the `cdx-remote-exec` service), so no `sudo` and no system-wide writes are needed.

Example (from codexis plugin):
```json
"postInstall": "install -d \"${HOME}/.local/bin\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-cli\" \"${HOME}/.local/bin/cdx-cli\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-link-rewriter\" \"${HOME}/.local/bin/cdx-link-rewriter\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-sledovane-dokumenty\" \"${HOME}/.local/bin/cdx-sledovane-dokumenty\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdxctl\" \"${HOME}/.local/bin/cdxctl\"",
"postUninstall": "rm -f \"${HOME}/.local/bin/cdx-cli\" \"${HOME}/.local/bin/cdx-link-rewriter\" \"${HOME}/.local/bin/cdx-sledovane-dokumenty\" \"${HOME}/.local/bin/cdxctl\"",
"onUpdate": "install -d \"${HOME}/.local/bin\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-cli\" \"${HOME}/.local/bin/cdx-cli\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-link-rewriter\" \"${HOME}/.local/bin/cdx-link-rewriter\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-sledovane-dokumenty\" \"${HOME}/.local/bin/cdx-sledovane-dokumenty\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdxctl\" \"${HOME}/.local/bin/cdxctl\""
```

For multi-binary plugins, prefer extracting these commands into `hooks/install-binaries.sh` and `hooks/uninstall-binaries.sh` (see the `codexis`, `cdx-sk`, `ares`, `cdx-at`, `cdx-cz-psp`, `cdx-cz-spp` plugins). Those scripts default `TARGET_BIN_DIR` to `${HOME}/.local/bin` and can be overridden via the `TARGET_BIN_DIR` env var if a different prefix is needed.

## Localization (i18n)

Every `marketplace.json` / `plugin.json` / `component.json` and every top-level
`SKILL.md` frontmatter carries an `i18n` block with localized display strings
for the CDX daemon UI. Technical identifiers (`name`, `id`) stay
kebab-case and are **never** translated — only user-facing strings are.

### JSON manifests

```json
{
  "name": "codexis",
  "description": "...",
  "tags": ["legal", "czech"],
  "i18n": {
    "cs": {
      "displayName": "CODEXIS — Česká legislativa",
      "description": "Přístup k databázi CODEXIS...",
      "tagLabels": { "legal": "Právo", "czech": "Česko" }
    },
    "en": { "displayName": "...", "description": "...", "tagLabels": { ... } },
    "sk": { "displayName": "...", "description": "...", "tagLabels": { ... } }
  }
}
```

For `component.json` the same pattern applies with `displayName` + `description`.

### Markdown frontmatter (`SKILL.md`, agents)

The top-level `description` stays as the **LLM trigger text** — never translate it.
User-facing short text lives under `i18n.<lang>.summary`.

```yaml
---
name: codexis
description: "This skill should be invoked whenever user needs Czech legal research..."   # AI trigger
version: 2.1.0
i18n:
  cs:
    displayName: "CODEXIS — Legislativa ČR"
    summary: "Vyhledávání v české a evropské legislativě."
  en:
    displayName: "CODEXIS — Czech Legislation"
    summary: "Search Czech and EU legislation."
  sk:
    displayName: "CODEXIS — Česká legislatíva"
    summary: "Vyhľadávanie v českej a európskej legislatíve."
---
```

Supported languages: `cs`, `en`, `sk`. Fallback chain in the daemon: `i18n[current] → i18n.en → description` (or technical `name` for titles).

## A2UI Visualization Schema

Visualization skills use the A2UI schema format:

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "chart|graph|grid|timeline|calendar|tree|flowchart|kanban|map",
  "title": { "literalString": "Title" },
  "config": { /* type-specific options */ },
  "data": { /* type-specific content */ }
}
```

Each visualization type has a SKILL.md (schema docs) and template.html (renderer) in `references/`.
