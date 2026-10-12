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
│   └── cdx-sledovane-dokumenty
└── src/
    ├── build.sh
    └── cdx-cli/
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
- Copies the git-tracked files, minus the exclusions in `dist-exclusions.txt` (including the root `README.md`)
- Overlays files from `dist-content/` into the dist root while preserving relative paths
- Commits the result as the only commit of a fresh git repository in `dist/`, dated like the source commit

Examples:

- `dist-content/README.md` becomes `dist/README.md`
- `dist-content/folder/test.txt` becomes `dist/folder/test.txt`

Run:

```bash
./create-dist.sh
```

## Tests

Run the suites through the single entry point:

```bash
python tests/run.py <unit|marketplace|eval|all>
```

## VM volume image

The distribution can be packaged as the read-only marketplace volume consumed by CODEXIS VMs:

```bash
./build-volume-image.sh \
  --source ./dist \
  --tag registry.agrp.dev/legal-machine/codexis-marketplace-volume:<dist-commit>
```

The source must be a clean distribution checkout. The resulting scratch OCI image contains a
world-readable `/volume.ext4`; the filesystem contains the distribution and its full Git revision
in `/.codexis-marketplace-revision`.
