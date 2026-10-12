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
| contract | the GraphQL documents a CLI sends (`plugins/<name>/src/<crate>/src/queries/*.graphql`) | central harness |
| conformance | the repo's own conventions: manifests, READMEs, i18n, hygiene | central harness |
| install | `plugins/<name>/tests/install/expected.json` (just the `skills` a plugin registers) | central harness |
| behavioral | `plugins/<name>/tests/promptfoo/*.config.yaml` (one config per scenario) | promptfoo |

The contract tier holds a CLI's GraphQL documents against the schema a live
daemon serves, so a field the daemon renames is caught before a user runs the
command. It asks for the schema once and validates locally, which is why it runs
ahead of the marketplace tier rather than inside it.

promptfoo is the single behavioral tier (the old homegrown e2e runner was
removed once every scenario had a promptfoo equivalent). Configs live per-plugin;
the shared machinery (`provider.py`, `assertions.py`, `run.py`) stays central in
`tests/promptfoo/` and each config reaches it by an explicit relative back-path
(`file://../../../../tests/promptfoo/...`). A config that uploads binary fixtures
keeps them in `plugins/<name>/tests/fixtures/`, referenced by a relative `source:`.

Every marketplace plugin must own an install contract
(`tests/install/expected.json`) naming **exactly** the skills under `skills/` -
`tests/test_conformance.py` fails the build otherwise. The contract is consumed
as an allowlist by the marketplace tier: a skill it omits is never install- or
uninstall-tested, and nothing else notices, so the two must not drift apart.
A behavioral (promptfoo) config is recommended but not enforced: the
eval tier is opt-in in CI, so requiring a config that does not routinely run adds
friction without a signal. Add one when a plugin's behavior is worth an
LLM-driven check; the requirement can return once the eval tier becomes a
routine gate.

Two different `tests/` folders exist: `tests/` at the repo root is the central
harness; `plugins/<name>/tests/` holds one plugin's own tests (unit, install,
promptfoo). Same name, different scope.

Inside the central one, each directory has one role: `tiers/` is what `run.py`
launches, `harness/` is what the tiers share, `unit/` tests the harness and the
tiers, and `conformance/` tests the repository itself. `promptfoo/` is the eval
tier, machinery and all.

### Running tests

Use the single entry point:
`python tests/run.py <check|test|contract|marketplace|eval|all>`. Tiers are cut
by the kind of failure they report, and happen to rise in cost down the list.

| tier | what it answers | what it runs |
| --- | --- | --- |
| `check` | does the source obey its rules? | ruff, ruff-format, ty, conformance, `cargo fmt`, `turbo lint`, `turbo typecheck` |
| `test` | does the code work? | pytest, `cargo test`, `turbo test` |
| `contract` | does the client still match the daemon? | the GraphQL documents against a live schema |
| `marketplace` | does every plugin install and uninstall? | the whole plugin lifecycle |
| `eval` | does the model behave? | promptfoo (opt-in) |

Lint and types sit in `check` rather than beside the tests deliberately: a lint
config broken outright still leaves `turbo test` green.

`check` runs the conformance tree and `test` runs everything else, so a bare
`pytest` runs both and matches neither tier's count. That is deliberate - a
developer wants the whole suite, a tier wants its own half.

Add `--strict` in CI, where the toolchains are provisioned: without it a missing
one is skipped rather than failing, and a stage that ran less than it should
still reports green. `test` is always strict, having nothing to auto-detect for.

`check` and `test` also take `--only backend|frontend`, which runs that part of
the tier alone. The split follows the toolchain, so a pipeline can give each
part its own stage, see which half broke, and run the two at the same time if it
wants. Without the flag a tier runs everything, which is what you want locally.

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
├── README.md             # User-facing overview (required, Czech)
└── shared/               # Shared resources
```

## Plugin README

Every plugin needs a `README.md`; `tests/test_conformance.py` fails the build
without one. It is the only text that tells a user what they just installed, and
it ships publicly (`dist-exclusions.txt` anchors `/README.md` to the repo root,
so `plugins/*/README.md` reaches the dist).

**Start at `## Co to je?` - no `# Title`, no tagline.** The daemon renders the
README on the plugin page underneath a header it builds from the manifest
(`i18n.<lang>.displayName` as the heading, `i18n.<lang>.description` as the
subtitle). A README that opens with its own title and one-liner prints the plugin
name twice and says the same thing twice. `##` is therefore the top level a
README may use.

Write it for the person using the plugin, not the person building it. Follow
`plugins/codexis/README.md`: `## Co to je?`, what the source contains,
`## K čemu se to hodí v praxi?`, and the official source. Add
`## Nastavení` when the user must supply a key themselves, `## Součásti pluginu`
when the plugin ships an app, and `## Doporučená kombinace` to point at plugins
that pair with it.

Two rules learned the hard way:

- **Never describe a capability you have not read in the plugin's own
  `SKILL.md` / `references/`.** The first draft claimed the Czech cadastre
  returns owner names; `references/api.md` says *"this API has no owners"*.
  A confident wrong claim reads as a broken plugin.
- **Keep out anything that ages on its own** - our libraries and models, exact
  counts ("seven courts"), schedules ("every morning at 7"). Name the capability,
  not the implementation. Defaults that only change by a deliberate edit (an
  e-mail toggle) are fine and worth stating.

The conformance test checks only that the file exists. Contents are a review
concern - pinning prose to manifests constrains how the README may be written
and catches only cosmetic drift.

## Example prompts

A skill's `SKILL.md` frontmatter holds ready-made prompts the daemon renders on
the plugin page, under the `i18n` block it already carries. They are **optional** -
`video-analyze` and `visualization` ship none on purpose, because nobody
addresses them directly; they run as part of another request.

```yaml
i18n:
  cs:
    displayName: "..."
    summary: "..."
    examplePrompts:
      - "Ověř v ARES firmu s IČO 27082440 - sídlo, aktivitu a jestli je plátcem DPH."
  en:
    ...
```

Prompts sit next to the `displayName` and `summary` they belong to, so a skill
carries its own user-facing text in one place. They are an ordered list of plain
strings - no `id` and no `title`. The whole prompt renders, so a separate label
would say the same thing twice and would let the chip advertise something other
than what it sends; ordering carries whatever a key used to. All three languages
are mandatory once any of them lists a prompt, and every language must list the
same number.

At most three prompts per language, and three is the target. They must cover
different capabilities rather than one question reworded. The two README rules
apply unchanged:

- **Never write a prompt for a capability you have not read in the plugin's own
  `SKILL.md` / `references/`.** Both cadastre plugins document that their API
  returns no owners; a "who owns this parcel" prompt would look like a broken
  plugin on the first click.
- **Keep out anything that ages on its own** - counts, deadlines, model names.

Two rules specific to prompts:

- **The whole text renders**, so keep it to one sentence of roughly 100
  characters.
- **A concrete value is only allowed when that record exists for everyone.**
  A real IČO is fine; an invented one returns nothing and reads as broken. Where
  the subject is inherently the user's own - their parcel, their proceeding,
  their data box, their internal directives - name the capability and let the
  assistant ask for the identifier. That still runs; a `«fill this in»`
  placeholder does not.

Put each prompt on the skill that actually serves it, and only on user-facing
skills: `codexis` also ships `cdxctl`, which administers platform objects and is
not something a user asks about.

`tests/test_conformance.py` validates the block when it exists and never requires
it. Em-dashes are not checked there - `tests/test_repo_hygiene.py` already sweeps
every tracked source file.

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
- **Write the behaviour, not its provenance.** State how the system behaves and what
  to do about it. How that was discovered, what the vendor's own docs claim, and why
  our code enforces it where it does are invisible to the reader and cost it tokens -
  they belong in the commit message and code comments, where a maintainer looks for
  them. A skill is not a changelog. Same rule bans describing the CLI's own leniency
  ("a lone condition is wrapped for you"): it is true, and the only thing it tells
  the reader is that it may be sloppy. State the required form instead.
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

A migration of user data in the VM (moving or rewriting files under `~/.cdx`) never lives in a plugin or its hooks: it is a cdx-daemon guest provision migration (`cdx-daemon-app/src/main/resources/provisioning/migrations/NNN-name.sh`), which runs once per VM start.

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

### Debian packages

`shared/deb/build-debs.sh` builds one package per plugin into `target/deb/`,
plus `codexis-marketplace`, which depends on all of them, and a `Packages`
index, so the directory is a flat apt repository. Packages are named
`codexis-<plugin>` (a `codexis*` plugin keeps its name) and versioned
`<plugin.json version>-1`. They hold the committed artifacts, so run
`./build.sh` first. `shared/deb/smoke-test.sh` installs `codexis-marketplace`
into a clean Ubuntu 26.04 and checks every package.

A package installs the shipped plugin tree (what the dist ships, minus the
lifecycle `hooks/*.sh`) under `/usr/share/cdx-plugins/codexis/<plugin>/` and
puts every `bin/` entry on `/usr/bin`. Its maintainer scripts keep
`/usr/share/cdx-plugins/codexis/plugins.txt` listing the installed plugin
directories.

Depends is derived from the contents (shared libraries, script shebangs) and
plugin.json `dependencies`. Everything else the plugin or its skills need at
runtime - a tool a binary spawns, a command the skills pipe through, a Python
library - is a hard dependency declared in `plugins/<name>/deb/control` (a
single `Depends` field). A pure-Python library Ubuntu does not package goes in
`plugins/<name>/deb/wheels` (`<url> sha256=<hex>`) and is unpacked into the
plugin's `lib/`. A script that becomes a command under another name goes in
`plugins/<name>/deb/links` (dh_link format). A Node bundle must not `require`
anything but `node:` built-ins.

### Source Previews (`previews/`)

A plugin can render an in-chat HTML preview for the sources it publishes. Publish sources with `previewUrl` in the `<plugin-name>://<resource-path>` form; the daemon resolves it through the plugin's preview handler declared by `previews/preview.json`:

```json
{ "id": "preview", "runtime": { "type": "cgi", "binary": "../bin/<cli>", "args": ["preview"] } }
```

The binary path is resolved relative to `previews/` and may point into the plugin's `bin/` (it must stay inside the plugin directory). The daemon executes it CGI-style in the user's VM with the resource path in `PATH_INFO` (+ `QUERY_STRING`); the handler prints a CGI response (`Status`/`Content-Type` headers, blank line, HTML body). Static assets placed next to the descriptor (e.g. `previews/preview.css`) are served as-is, so the page can reference them relatively (`<link rel="stylesheet" href="./preview.css">`). The rendered HTML drives the chat panel: `<title>` becomes the panel title and an optional `<link rel="canonical" href="…">` becomes the "open in new tab" target. See the `codexis` plugin (`cdx-cli preview`) for the reference implementation.

A handler that writes preview files to disk must never derive the target path itself. The daemon owns the filesystem layout and passes it in the CGI environment:

| ENV | Meaning |
| --- | --- |
| `PREVIEW_DIR` | absolute directory the handler writes generated preview files into (already scoped to the chat) |
| `PREVIEW_ASSET_BASE_URL` | URL prefix the daemon serves the plugin's `previews/` assets from, e.g. `<PREVIEW_ASSET_BASE_URL>/preview.css` |

A file-writing handler must fail (`Status: 5xx`) when either is missing rather than falling back to a hardcoded path. `PREVIEW_ASSET_BASE_URL` is needed because such a page is opened through the file preview renderer, not from the plugin endpoint, so a relative `./preview.css` would not resolve.

### Frontend changes need a component rebuild

`frontend/apps/<app>/` is **source**. The daemon never runs it. What the daemon
serves is the built copy committed under `plugins/<plugin>/components/<app>/`
(hashed JS/CSS in `assets/`, plus its own `locales/`). Change the source without
rebuilding and the dev server shows your change while the daemon keeps serving
the old bundle - the change looks done and is not.

After touching anything under `frontend/`, rebuild the affected plugins and
commit the result. The builder is `plugins/<plugin>/components/build.sh`; it
needs docker, builds the app in a pinned `oven/bun` image and syncs `dist/`
into the component directory.

Which plugin ships which app is not a fixed list - one plugin can ship several
apps, and the set changes. Ask the repo instead of remembering:

```bash
# which plugin ships app <app>
grep -l "apps/<app>" plugins/*/components/build.sh
```

A change under `frontend/packages/` reaches **every** app, so rebuild all of
them:

```bash
for b in plugins/*/components/build.sh; do "$b"; done
```

When in doubt, rebuild everything - it is cheap and a stale bundle is silent.

The generated filenames carry a content hash and therefore change on every
rebuild - that is the point, see the caching rules above. Expect the diff to
delete the old `index-<hash>.js` and add a new one.

## Display names and versions

### `displayName`: add a qualifier only when the name needs one

Two shapes are in use, and which one applies follows from the name itself:

- **Opaque name → `Brand - what it is`.** A brand or acronym a user cannot
  decode: `CODEXIS - Právní databáze`, `ARES - registr ekonomických subjektů`,
  `ONEPOST - datové schránky`, `SÚKL - registr léčiv`.
- **Self-describing name → leave it alone.** Adding a qualifier would only
  repeat it: `Katastr nemovitostí ČR`, `Hlídač insolvencí ČR`, `Vizualizace dat`.

**Take the qualifier from how the product describes itself**, not from what its
API happens to expose. Datify's API is collections and datasets, which reads
like a "data platform" - but Datify calls itself a *digitální AI kancelář*, and
that is what belongs in the name. Check the vendor's own site before inventing
a label; a wrong one ships to every user and is awkward to change later.

Keep `displayName` in step across all three places that carry it:
`marketplace.json`, `plugin.json` and the skill's `SKILL.md` frontmatter.

### Versions start at 1.0.0

A plugin that ships is `1.0.0`, not `0.1.0` - the marketplace has no pre-release
channel, so anything installable is released. Bump `version` in `plugin.json`
and, for Rust plugins, the crate's `Cargo.toml` in the same commit: the CLI
reports it (`--version`) and it usually reaches the API as the `User-Agent`.

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
