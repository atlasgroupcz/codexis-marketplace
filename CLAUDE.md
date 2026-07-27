# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Text conventions

Never write em-dashes or en-dashes (the long typographic dashes, Unicode U+2014
and U+2013). Use a plain hyphen (`-`) instead.
This applies everywhere: plugin display names, descriptions, i18n labels,
component UI text, SKILL docs, code comments, and commit messages. Em-dashes had
crept into user-facing names and looked unprofessional, so the repo was
normalized to hyphens - keep it that way.

## Testing

### Two kinds of test

- **Code tests** exercise a plugin's own code and run under a framework:
  `test_*.py` (pytest) for Python, `#[cfg(test)]` inline + `src/<crate>/tests/*.rs`
  (cargo) for Rust.
- **Scenario specs** are declarative data driven by the central harness:
  `expected.json` (marketplace/install contract) and
  `tests/promptfoo/*.config.yaml` (behavioral evals), both per-plugin.

### Taxonomy

| Tier | Location | Runner |
| --- | --- | --- |
| unit | Python: `plugins/<name>/tests/unit/` (pytest); Rust: inline `#[cfg(test)]` in `src/<crate>/src/` | pytest / cargo |
| integration | Rust: `plugins/<name>/src/<crate>/tests/*.rs`; Python: `tests/integration/` if needed | cargo / pytest |
| install | `plugins/<name>/tests/install/expected.json` (just the `skills` a plugin registers) | central harness |
| behavioral | `plugins/<name>/tests/promptfoo/*.config.yaml` (one config per scenario) | promptfoo |

promptfoo is the single behavioral tier (the old homegrown e2e runner was
removed once every scenario had a promptfoo equivalent). Configs live per-plugin;
the shared machinery (`provider.py`, `assertions.py`, `run.py`) stays central in
`tests/promptfoo/` and each config reaches it by an explicit relative back-path
(`file://../../../../tests/promptfoo/...`). A config that uploads binary fixtures
keeps them in `plugins/<name>/tests/fixtures/`, referenced by a relative `source:`.

Every marketplace plugin must own an install contract
(`tests/install/expected.json`) - `tests/test_conformance.py` fails the build
otherwise. A behavioral (promptfoo) config is recommended but not enforced: the
eval tier is opt-in in CI, so requiring a config that does not routinely run adds
friction without a signal. Add one when a plugin's behavior is worth an
LLM-driven check; the requirement can return once the eval tier becomes a
routine gate.

Two different `tests/` folders exist: `tests/` at the repo root is the central
harness; `plugins/<name>/tests/` holds one plugin's own tests (unit, install,
promptfoo). Same name, different scope.

### Running tests

Use the single entry point: `python tests/run.py <unit|marketplace|eval|all>`.

### Coverage

New or changed plugin code must reach >=90% line coverage on the changed code,
verified from the coverage report (`pytest --cov` for Python, `cargo llvm-cov`
for Rust). Checked in review; not an automated CI threshold.

### Test-quality rules

General principles, independent of language or framework:

- Name each test after the behavior it checks; one behavior per test.
- Cover the happy path first; add an error/negative test only when it catches
  something real, not mechanically.
- Every test must assert real behavior. A test that only builds a value or
  restates the implementation is not a test - if it cannot fail, delete it.
- Use real dependencies where you can run them; mock only what you cannot
  (external HTTP APIs, the daemon, third-party services). Never mock the thing
  under test.
- On any failure (test, eval, pipeline), find the cause before changing code -
  no shotgun fixes.
- Before calling something green, show the command output - do not assert it
  passed.

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
├── workflow-templates/   # Workflow scripts the chat can fetch and adapt (*.js)
└── shared/               # Shared resources
```

## Workflow templates

`workflow-templates/*.js` are ready-made multi-phase workflows the daemon offers the model when
it reaches for the `workflow` tool. Each file is a normal workflow script - `export const meta`
plus `export default async function (args)` - with a header comment that decides how it is
advertised:

```js
// @template legal-research
// @use-when A legal question that must be answered from the law and applied to the client's
//   own documents.
// @jurisdictions CZ SK EU
```

`@use-when` is required and is the only thing the model sees before fetching the template, so
write it as the selection hint, not as a summary. `@template` defaults to the file name and
`@jurisdictions` ranks the template for chats in that jurisdiction. A file without `@use-when`
is skipped. Templates are never run as they are: the model fetches the source, adapts it, and
submits the adapted script, so keep them well under the 32 000-character script limit.

## Writing skills - Anthropic conventions

Follow Anthropic's skill-authoring best practices (platform.claude.com, Agent Skills >
Skill authoring best practices) for every SKILL.md in this repo:

- **The description decides triggering.** Write it in third person ("Manages...", never
  "I can..." / "You can..."), state both WHAT the skill does and WHEN to use it ("Use
  when..."), and include the trigger keywords users actually say. Max 1024 characters,
  no XML tags. If a skill fails to trigger, fix the description, not the body.
- **Name**: max 64 chars, lowercase letters/digits/hyphens; prefer gerund or noun-phrase
  names (`processing-pdfs`, `pdf-processing`); never vague (`helper`, `utils`).
- **Concise body.** Claude is already smart - add only what it does not know, and make
  every paragraph justify its token cost. Keep the SKILL.md body under 500 lines; push
  depth into `references/*.md` linked ONE level deep from SKILL.md (no reference file
  pointing at another reference file). Reference files over 100 lines start with a table
  of contents.
- **One default, not a menu.** Recommend a single tool/approach with an escape hatch for
  the exception, instead of listing alternatives.
- **Concrete examples** (input -> output pairs) beat abstract descriptions. For multi-step
  workflows give numbered steps or a checklist; for quality-critical steps add a
  validation loop (run the check, fix, re-run).
- **Consistent terminology**: pick one term per concept and use it everywhere.
- **No time-sensitive content.** No "before/after <date>" branches; keep legacy behavior
  in a clearly marked deprecated/old-patterns note.
- **Match specificity to fragility**: exact commands for fragile one-way operations,
  general guidance where many approaches work.

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
  "iconPath": "icon.svg",
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
  "skills": "./skills",
  "env": {
    "CODEXIS_PLUGIN_<PLUGIN>_<NAME>": "https://example/api"
  }
}
```

### Icons

Every manifest **declares** its icon path; the daemon no longer scans the repo
for `icon.svg`/`icon.png`/etc. Without a declaration the UI renders initials.

- `plugin.json` → `iconPath` (or `icon`), resolved relative to the **plugin
  directory** (e.g. `"icon.svg"` for `plugins/<name>/icon.svg`).
- `marketplace.json` → `iconPath`, resolved relative to the **marketplace root**.
- `component.json` → `icon`, resolved relative to the **component directory**
  (e.g. `"assets/icon.png"`).
- `SKILL.md` / agent `*.md` frontmatter → `icon:`, resolved relative to the
  **skill/agent directory**.

### Plugin-owned environment variables

Declare plugin-owned env vars in `plugin.json` under `env`. The host merges these into every shell process the plugin runs. Do not write env files; do not set them in install hooks.

Naming convention:
- `CODEXIS_PLUGIN_*` - plugin-owned configuration (URLs, IDs). Each key belongs to exactly one plugin; do not redeclare another plugin's key.
- `CODEXIS_USER_*` - per-user secrets injected by the host (Codexis API tokens, LiteLLM keys). Plugins are read-only consumers; never log, never embed in builds.
- `CODEXIS_PUBLIC_*` - host-injected runtime context (daemon URL, user home, session id). Plugins are read-only consumers.

### Lifecycle Hooks

Plugins support three lifecycle hooks executed as shell commands by the harness:

- **`postInstall`** - runs after the plugin is installed
- **`postUninstall`** - runs after the plugin is uninstalled
- **`onUpdate`** - runs after the plugin is updated

The variable `${PLUGIN_DIR}` is available in hook commands and resolves to the plugin's installation directory.

### Executables / Binaries

Plugin executables must be placed in a `bin/` folder inside the plugin directory. Lifecycle hooks are typically used to install these binaries into the user-local `${HOME}/.local/bin/` location on install/update and remove them on uninstall. The cdx guest VM has `${HOME}/.local/bin` on `PATH` (both for interactive login shells and for the `cdx-remote-exec` service), so no `sudo` and no system-wide writes are needed.

Executables don't have to be native binaries - the guest VM ships `python3`
and `nodejs` from apt, so interpreted entrypoints are fine (`katastr-cli`,
`insolvence-cli` are Python; `sid-cli` is bundled TypeScript).

**TypeScript plugins** (pattern: `plugins/sid`): source lives in
`plugins/<name>/src/<cli>/` as a bun project; `src/build.sh` bundles it
inside a digest-pinned `oven/bun` image into ONE self-contained file in
`bin/`, committed like any other artifact. No `node_modules` at runtime.
Two constraints learned the hard way:

- **Bundle as CJS** (`bun build --target=node --format=cjs`). Artifacts are
  installed without an extension (`sid-cli`), and Node loads an
  extensionless file as CommonJS - an ESM bundle dies with "Cannot use
  import statement outside a module".
- **Don't prepend a shebang** in the build script if the entrypoint source
  already has one; bun preserves it, and a doubled shebang is invalid JS.

Run the container with `--user "$(id -u):$(id -g)"`, otherwise `dist/` and
`bin/` end up root-owned and the host user can't even delete them.

Example (from codexis plugin):
```json
"postInstall": "install -d \"${HOME}/.local/bin\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-cli\" \"${HOME}/.local/bin/cdx-cli\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-sledovane-dokumenty\" \"${HOME}/.local/bin/cdx-sledovane-dokumenty\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdxctl\" \"${HOME}/.local/bin/cdxctl\"",
"postUninstall": "rm -f \"${HOME}/.local/bin/cdx-cli\" \"${HOME}/.local/bin/cdx-sledovane-dokumenty\" \"${HOME}/.local/bin/cdxctl\"",
"onUpdate": "install -d \"${HOME}/.local/bin\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-cli\" \"${HOME}/.local/bin/cdx-cli\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdx-sledovane-dokumenty\" \"${HOME}/.local/bin/cdx-sledovane-dokumenty\" && install -m 0755 \"${PLUGIN_DIR}/bin/cdxctl\" \"${HOME}/.local/bin/cdxctl\""
```

For multi-binary plugins, prefer extracting these commands into `hooks/install-binaries.sh` and `hooks/uninstall-binaries.sh` (see the `codexis`, `cdx-sk`, `ares`, `cdx-at`, `cdx-cz-psp`, `cdx-cz-spp` plugins). Those scripts default `TARGET_BIN_DIR` to `${HOME}/.local/bin` and can be overridden via the `TARGET_BIN_DIR` env var if a different prefix is needed.

### Source Previews (`previews/`)

A plugin can render an in-chat HTML preview for the sources it publishes. Publish sources with `previewUrl` in the `<plugin-name>://<resource-path>` form; the daemon resolves it through the plugin's preview handler declared by `previews/preview.json`:

```json
{ "id": "preview", "runtime": { "type": "cgi", "binary": "../bin/<cli>", "args": ["preview"] } }
```

The binary path is resolved relative to `previews/` and may point into the plugin's `bin/` (it must stay inside the plugin directory). The daemon executes it CGI-style in the user's VM with the resource path in `PATH_INFO` (+ `QUERY_STRING`); the handler prints a CGI response (`Status`/`Content-Type` headers, blank line, HTML body). Static assets placed next to the descriptor (e.g. `previews/preview.css`) are served as-is, so the page can reference them relatively (`<link rel="stylesheet" href="./preview.css">`). The rendered HTML drives the chat panel: `<title>` becomes the panel title and an optional `<link rel="canonical" href="…">` becomes the "open in new tab" target. See the `codexis` plugin (`cdx-cli preview`) for the reference implementation.

## Localization (i18n)

Every `marketplace.json` / `plugin.json` / `component.json` and every top-level
`SKILL.md` frontmatter carries an `i18n` block with localized display strings
for the CDX daemon UI. Technical identifiers (`name`, `id`) stay
kebab-case and are **never** translated - only user-facing strings are.

### JSON manifests

```json
{
  "name": "codexis",
  "description": "...",
  "tags": ["legal", "czech"],
  "i18n": {
    "cs": {
      "displayName": "CODEXIS - Česká legislativa",
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

The top-level `description` stays as the **LLM trigger text** - never translate it.
User-facing short text lives under `i18n.<lang>.summary`.

```yaml
---
name: codexis
description: "This skill should be invoked whenever user needs Czech legal research..."   # AI trigger
version: 2.1.0
i18n:
  cs:
    displayName: "CODEXIS - Legislativa ČR"
    summary: "Vyhledávání v české a evropské legislativě."
  en:
    displayName: "CODEXIS - Czech Legislation"
    summary: "Search Czech and EU legislation."
  sk:
    displayName: "CODEXIS - Česká legislatíva"
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
