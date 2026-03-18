#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${PROJECT_DIR}/schemas/search"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

readonly API_DOCS_URLS=(
  "https://beta.next.codexis.cz/rest/v3/api-docs"
  "https://beta.next.codexis.cz/v3/api-docs"
)

declare -A INPUT_ROOTS=(
  [ALL]="CdxSearchRequest"
  [COMMENT]="CommentSearchRequest"
  [CR]="CrSearchRequest"
  [ES]="EsSearchRequest"
  [EU]="EuSearchRequest"
  [JD]="JdSearchRequest"
  [LT]="LtSearchRequest"
  [SK]="SkSearchRequest"
  [VS]="VsSearchRequest"
)

declare -A OUTPUT_ROOTS=(
  [ALL]="GlobalSearchResponse"
  [COMMENT]="CommentSearchResponse"
  [CR]="CrSearchResponse"
  [ES]="EsSearchResponse"
  [EU]="EuSearchResponse"
  [JD]="JdSearchResponse"
  [LT]="LtSearchResponse"
  [SK]="SkSearchResponse"
  [VS]="VsSearchResponse"
)

API_DOCS_PATH="${TMP_DIR}/api-docs.json"
API_DOCS_URL=""

fetch_api_docs() {
  local url
  for url in "${API_DOCS_URLS[@]}"; do
    if curl -fsSL "${url}" -o "${API_DOCS_PATH}"; then
      API_DOCS_URL="${url}"
      return 0
    fi
  done

  echo "Failed to fetch API docs from known locations." >&2
  return 1
}

collect_schema_names() {
  local root_name="$1"
  local -a queue=("${root_name}")
  local -A seen=()

  while ((${#queue[@]} > 0)); do
    local current="${queue[0]}"
    queue=("${queue[@]:1}")

    if [[ -n "${seen[${current}]:-}" ]]; then
      continue
    fi
    seen["${current}"]=1

    mapfile -t deps < <(
      jq -r --arg name "${current}" '
        .components.schemas[$name]
        | if . == null then
            empty
          else
            (
              .. | objects | .["$ref"]? // empty
              | select(startswith("#/components/schemas/"))
              | sub("^#/components/schemas/"; "")
            )
          end
      ' "${API_DOCS_PATH}" | sort -u
    )

    local dep
    for dep in "${deps[@]}"; do
      if [[ -n "${dep}" && -z "${seen[${dep}]:-}" ]]; then
        queue+=("${dep}")
      fi
    done
  done

  printf '%s\n' "${!seen[@]}" | sort
}

write_bundle() {
  local source_code="$1"
  local kind="$2"
  local root_name="$3"
  local target_path="$4"
  local names_json="${TMP_DIR}/${source_code,,}-${kind}-names.json"

  collect_schema_names "${root_name}" | jq -R . | jq -s . > "${names_json}"

  jq \
    --arg source "${source_code}" \
    --arg kind "${kind}" \
    --arg root "${root_name}" \
    --arg apiDocsUrl "${API_DOCS_URL}" \
    --arg fetchedAt "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
    --slurpfile names "${names_json}" \
    '{
      source: $source,
      kind: $kind,
      root: $root,
      apiDocsUrl: $apiDocsUrl,
      fetchedAt: $fetchedAt,
      components: (
        .components.schemas
        | with_entries(select(.key as $key | $names[0] | index($key)))
      )
    }' "${API_DOCS_PATH}" > "${target_path}"
}

main() {
  fetch_api_docs
  mkdir -p "${OUTPUT_DIR}"

  local source_code
  for source_code in ALL COMMENT CR ES EU JD LT SK VS; do
    mkdir -p "${OUTPUT_DIR}/${source_code}"
    write_bundle \
      "${source_code}" \
      "input" \
      "${INPUT_ROOTS[${source_code}]}" \
      "${OUTPUT_DIR}/${source_code}/input.bundle.json"
    write_bundle \
      "${source_code}" \
      "output" \
      "${OUTPUT_ROOTS[${source_code}]}" \
      "${OUTPUT_DIR}/${source_code}/output.bundle.json"
  done

  echo "Updated search schema bundles from ${API_DOCS_URL}"
}

main "$@"
