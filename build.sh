#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Shared build config (pinned Docker image, platform, target dir) + the
# per-plugin install helpers. Sourcing sets REPO_ROOT, RUST_IMAGE,
# CARGO_BUILD_ARGS and _CDX_DOCKER_PLATFORM used by build_rust_workspace, and
# exposes build_plugin_crate / install_workspace_binary to the plugin scripts.
source "$ROOT_DIR/shared/scripts/plugin-build.sh"

# Compile every Rust crate ONCE (all plugin CLIs + the shared libs) in a single
# `cargo build --workspace`, so each shared dependency (serde, ureq, reqwest, ...)
# is compiled once instead of once per plugin. Each plugin's src/build.sh then
# just installs its binary from the shared target/ - no per-plugin compile. The
# produced binaries are unchanged: static, self-contained.
build_rust_workspace() {
  echo "==> Building the entire Rust workspace for $_CDX_DOCKER_PLATFORM (one pass)"
  docker run --rm \
    --platform "$_CDX_DOCKER_PLATFORM" \
    -u "$(id -u):$(id -g)" \
    -e CARGO_HOME=/tmp/cargo \
    -v "$ROOT_DIR:/workspace" \
    -w /workspace \
    "$RUST_IMAGE" \
    cargo build --workspace $CARGO_BUILD_ARGS
}

# 1. Compile all Rust crates once. The flag tells the plugin scripts that target/
#    is current for this invocation, so they install without recompiling; run on
#    their own they compile first (see _ensure_crate_built).
build_rust_workspace
export CDX_WORKSPACE_BUILT=1

# 2. Run every plugin's build.sh. Rust plugins install their binary from the
#    shared target/ (build_plugin_crate, no per-plugin compile); non-Rust plugins
#    (Python / TypeScript) build as before.
#    This loop is the one build step that cannot run in a toolchain image: it
#    drives both the Rust and the frontend scripts, so neither image can host
#    it. It therefore runs on the host and has to keep to what every host has -
#    which rules out `mapfile`, a bash 4 builtin absent from the bash 3.2 macOS
#    still ships.
built=0
while IFS= read -r script; do
  echo "==== Running ${script#"$ROOT_DIR"/} ===="
  "$script"
  built=$((built + 1))
done < <(
  find "$ROOT_DIR/plugins" -mindepth 3 -maxdepth 3 -type f \
    \( -path '*/components/build.sh' -o -path '*/src/build.sh' \) \
    | sort
)

if (( built == 0 )); then
  echo "ERROR: No plugin build scripts found under plugins/*/{components,src}/build.sh" >&2
  exit 1
fi

echo "All plugin builds completed successfully."
