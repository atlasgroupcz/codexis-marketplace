#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Shared build config (pinned Docker image, arch, target dir) + the per-plugin
# install helpers. Sourcing sets REPO_ROOT, RUST_IMAGE, CARGO_BUILD_ARGS,
# _CDX_DOCKER_PLATFORM and _CDX_MACOS_ARM64_TARGET used by build_rust_workspace,
# and exposes build_plugin_crate / install_workspace_binary to the plugin scripts.
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

# 1. Compile all Rust crates once.
build_rust_workspace

# 2. Run every plugin's build.sh. Rust plugins install their binary from the
#    shared target/ (build_plugin_crate, no per-plugin compile); non-Rust plugins
#    (Python / TypeScript) build as before.
mapfile -t BUILD_SCRIPTS < <(
  find "$ROOT_DIR/plugins" -mindepth 3 -maxdepth 3 -type f \
    \( -path '*/components/build.sh' -o -path '*/src/build.sh' \) \
    | sort
)

if (( ${#BUILD_SCRIPTS[@]} == 0 )); then
  echo "ERROR: No plugin build scripts found under plugins/*/{components,src}/build.sh" >&2
  exit 1
fi

for script in "${BUILD_SCRIPTS[@]}"; do
  rel_script="${script#$ROOT_DIR/}"
  echo "==== Running $rel_script ===="
  "$script"
done

echo "All plugin builds completed successfully."
