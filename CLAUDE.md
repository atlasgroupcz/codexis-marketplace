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

Plugin executables must be placed in a `bin/` folder inside the plugin directory. Lifecycle hooks are typically used to copy these binaries to a system PATH location (e.g. `/usr/local/bin/`) on install/update and remove them on uninstall.

Example (from codexis plugin):
```json
"postInstall": "sudo cp \"${PLUGIN_DIR}/bin/cdx-cli\" /usr/local/bin/cdx-cli && sudo cp \"${PLUGIN_DIR}/bin/cdx-link-rewriter\" /usr/local/bin/cdx-link-rewriter && sudo cp \"${PLUGIN_DIR}/bin/cdx-sledovane-dokumenty\" /usr/local/bin/cdx-sledovane-dokumenty && sudo cp \"${PLUGIN_DIR}/bin/cdxctl\" /usr/local/bin/cdxctl",
"postUninstall": "sudo rm -f /usr/local/bin/cdx-cli /usr/local/bin/cdx-link-rewriter /usr/local/bin/cdx-sledovane-dokumenty /usr/local/bin/cdxctl",
"onUpdate": "sudo cp \"${PLUGIN_DIR}/bin/cdx-cli\" /usr/local/bin/cdx-cli && sudo cp \"${PLUGIN_DIR}/bin/cdx-link-rewriter\" /usr/local/bin/cdx-link-rewriter && sudo cp \"${PLUGIN_DIR}/bin/cdx-sledovane-dokumenty\" /usr/local/bin/cdx-sledovane-dokumenty && sudo cp \"${PLUGIN_DIR}/bin/cdxctl\" /usr/local/bin/cdxctl"
```

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
