#!/usr/bin/env bash
# Uploads every screenshot and video from the evidence directory to the GitLab project and prints
# the markdown to paste into the MR's Evidence section. The files never enter the repository.
set -euo pipefail

usage() {
  echo "Usage: bash upload-evidence.sh [evidence-dir]   (default: \$CDX_EVIDENCE_DIR or \$TMPDIR/cdx-evidence)" >&2
  exit 2
}

main() {
  [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage
  local dir="${1:-${CDX_EVIDENCE_DIR:-${TMPDIR:-/tmp}/cdx-evidence}}"
  local host="gitlab.agrp.dev"
  local project="profidata%2Fcodexis-marketplace"
  command -v glab >/dev/null || { echo "glab is required" >&2; exit 1; }
  command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }
  [[ -d "$dir" ]] || { echo "No evidence directory: $dir" >&2; exit 1; }

  local token
  token="$(glab config get token --host "$host")"
  [[ -n "$token" ]] || { echo "No glab token for $host - run: glab auth login --hostname $host" >&2; exit 1; }

  local file found=0
  for file in "$dir"/*.png "$dir"/*.webm "$dir"/*.mp4; do
    [[ -f "$file" ]] || continue
    found=1
    curl -fsS -H "PRIVATE-TOKEN: $token" -F "file=@$file" \
      "https://$host/api/v4/projects/$project/uploads" | jq -r '.markdown'
  done
  [[ "$found" == 1 ]] || { echo "No .png/.webm/.mp4 files in $dir" >&2; exit 1; }
}

main "$@"
