#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
RESOURCE_OUTPUT_DIR="${PROJECT_DIR}/schemas/resource"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

if [[ -n "${CODEXIS_API_DOCS_URL:-}" ]]; then
  readonly API_DOCS_URLS=("${CODEXIS_API_DOCS_URL}")
else
  readonly API_DOCS_URLS=(
    "https://beta.next.codexis.cz/rest/v3/api-docs"
    "https://beta.next.codexis.cz/v3/api-docs"
  )
fi

declare -A RESOURCE_DOC_PATHS=(
  [meta]="/rest/cdx-api/doc/{docId}/meta"
  [text]="/rest/cdx-api/doc/{docId}/text"
  [toc]="/rest/cdx-api/doc/{docId}/toc"
  [versions]="/rest/cdx-api/doc/{docId}/versions"
  [related]="/rest/cdx-api/doc/{docId}/related"
  [related-counts]="/rest/cdx-api/doc/{docId}/related/counts"
)

declare -A RESOURCE_CZ_LAW_PATHS=(
  [meta]="/rest/cdx-api/cz_law/{lawNumber}/{lawYear}/meta"
  [text]="/rest/cdx-api/cz_law/{lawNumber}/{lawYear}/text"
  [toc]="/rest/cdx-api/cz_law/{lawNumber}/{lawYear}/toc"
  [versions]="/rest/cdx-api/cz_law/{lawNumber}/{lawYear}/versions"
  [related]="/rest/cdx-api/cz_law/{lawNumber}/{lawYear}/related"
  [related-counts]="/rest/cdx-api/cz_law/{lawNumber}/{lawYear}/related/counts"
)

API_DOCS_PATH="${TMP_DIR}/api-docs.json"
API_DOCS_URL=""
FETCHED_AT=""

fetch_api_docs() {
  local url
  for url in "${API_DOCS_URLS[@]}"; do
    if curl -fsSL "${url}" -o "${API_DOCS_PATH}"; then
      API_DOCS_URL="${url}"
      FETCHED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
      return 0
    fi
  done

  echo "Failed to fetch API docs from known locations." >&2
  return 1
}

collect_schema_names_from_seeds() {
  local -a queue=("$@")
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

collect_schema_names_from_fragment() {
  local fragment_path="$1"
  local -a seeds=()

  mapfile -t seeds < <(
    jq -r '
    .. | objects | .["$ref"]? // empty
    | select(startswith("#/components/schemas/"))
    | sub("^#/components/schemas/"; "")
  ' "${fragment_path}" | sort -u
  )

  if ((${#seeds[@]} == 0)); then
    return 0
  fi

  collect_schema_names_from_seeds "${seeds[@]}"
}

write_resource_bundle() {
  local endpoint="$1"
  local target_path="$2"
  local operations_lines="${TMP_DIR}/${endpoint}-operations.ndjson"
  local operations_json="${TMP_DIR}/${endpoint}-operations.json"
  local names_json="${TMP_DIR}/${endpoint}-names.json"
  local patterns_lines="${TMP_DIR}/${endpoint}-patterns.ndjson"
  local patterns_json="${TMP_DIR}/${endpoint}-patterns.json"

  : > "${operations_lines}"
  : > "${patterns_lines}"

  local rest_path
  for rest_path in "${RESOURCE_DOC_PATHS[${endpoint}]}" "${RESOURCE_CZ_LAW_PATHS[${endpoint}]}"; do
    if ! jq -e --arg path "${rest_path}" '.paths[$path].get != null' "${API_DOCS_PATH}" >/dev/null; then
      continue
    fi

    jq -c \
      --arg path "${rest_path}" \
      '
        .paths[$path].get
        | if . == null then
            error("missing resource operation for path " + $path)
          else
            {
              path: $path,
              method: "get",
              summary,
              description,
              operationId,
              parameters: (.parameters // []),
              responses: (.responses // {})
            }
          end
      ' "${API_DOCS_PATH}" >> "${operations_lines}"

    case "${rest_path}" in
      /rest/cdx-api/doc/*)
        jq -cn --arg endpoint "${endpoint}" '"cdx://doc/<DOC_ID>/" + $endpoint' >> "${patterns_lines}"
        ;;
      /rest/cdx-api/cz_law/*)
        jq -cn --arg endpoint "${endpoint}" '"cdx://cz_law/<NUM>/<YEAR>/" + $endpoint' >> "${patterns_lines}"
        ;;
    esac
  done

  if [[ ! -s "${operations_lines}" ]]; then
    echo "Missing resource operations for endpoint ${endpoint}" >&2
    return 1
  fi

  jq -s . "${operations_lines}" > "${operations_json}"
  jq -s . "${patterns_lines}" > "${patterns_json}"

  collect_schema_names_from_fragment "${operations_json}" | jq -R . | jq -s . > "${names_json}"

  jq \
    --arg endpoint "${endpoint}" \
    --arg apiDocsUrl "${API_DOCS_URL}" \
    --arg fetchedAt "${FETCHED_AT}" \
    --slurpfile operations "${operations_json}" \
    --slurpfile names "${names_json}" \
    --slurpfile patterns "${patterns_json}" \
    '{
      endpoint: $endpoint,
      kind: "resource",
      format: "openapi-fragment",
      apiDocsUrl: $apiDocsUrl,
      fetchedAt: $fetchedAt,
      cdxPatterns: $patterns[0],
      operations: $operations[0],
      components: {
        schemas: (
          .components.schemas
          | with_entries(select(.key as $key | $names[0] | index($key)))
        )
      }
    }' "${API_DOCS_PATH}" > "${target_path}"
}

main() {
  fetch_api_docs
  mkdir -p "${RESOURCE_OUTPUT_DIR}"

  local endpoint
  for endpoint in meta text toc versions related related-counts; do
    write_resource_bundle \
      "${endpoint}" \
      "${RESOURCE_OUTPUT_DIR}/${endpoint}.bundle.json"
  done

  echo "Updated resource schema bundles from ${API_DOCS_URL}"
}

main "$@"
