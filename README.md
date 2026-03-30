# Build Layout and Conventions

This repository uses per-plugin build scripts plus one master build script.

## Source and Build Script Placement

For every plugin that has native code:

- Put source crates under `plugins/<plugin>/src/<crate>/`
- Put plugin build entrypoint at `plugins/<plugin>/src/build.sh`
- Put build output binaries into `plugins/<plugin>/bin/`

Example (`codexis`):

```text
plugins/codexis/
├── bin/
│   ├── cdx-cli
│   ├── cdx-sledovane-dokumenty
│   └── cdx-link-rewriter
└── src/
    ├── build.sh
    ├── cdx-cli/
    │   ├── Cargo.toml
    │   └── src/main.rs
    └── cdx-link-rewriter/
        ├── Cargo.toml
        └── src/main.rs
```

## Per-Plugin `src/build.sh` Contract

Each `plugins/<plugin>/src/build.sh` should:

- Be executable and fail on error (`set -euo pipefail`)
- Build **Linux only**, for the **same architecture as the current machine**
- Use **Dockerized, reproducible builds** (no host-native compilation)
- Build Rust crates with `cargo build --release --locked`
- Copy resulting executables to `plugins/<plugin>/bin/<binary-name>`
- Exit non-zero on any failure

## Master Build Script

Root script: `./build.sh`

Behavior:

- Discovers scripts at `plugins/*/src/build.sh`
- Runs them in sorted order, one by one
- Stops immediately on first failure
- Exits non-zero if any plugin build fails

Run all builds:

```bash
./build.sh
```

Run only `codexis` plugin build:

```bash
./plugins/codexis/src/build.sh
```

## Distribution Copy

Root script: `./create-dist.sh`

Behavior:

- Creates a gitignored `./dist/` directory inside this repository
- Recreates that directory from scratch on every run
- Builds the dist contents with `rsync` in a temporary staging directory
- Applies exclusions from `dist-exclusions.txt`, including the root `README.md`
- Overlays files from `dist-content/` into the dist root while preserving relative paths
- Treats `dist/` as a checkout of `git@github.com:atlasgroupcz/codexis-marketplace.git`
- Pushes the generated contents to the target branch after committing any detected changes

Branch selection:

- Uses `DIST_BRANCH` when set
- Otherwise uses the remote default branch when it can be resolved
- Falls back to `main` when the remote is empty or has no symbolic `HEAD`

Examples:

- `dist-content/README.md` becomes `dist/README.md`
- `dist-content/folder/test.txt` becomes `dist/folder/test.txt`

Run:

```bash
./create-dist.sh
```
