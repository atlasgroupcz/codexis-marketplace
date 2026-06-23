---
uuid: 74ddfc4a-3218-4262-9fd5-73d5ec8c886b
name: data-gouv-fr
icon: icon.svg
description: This skill should be invoked whenever the user needs facts, figures, or official information about France or French public affairs — demographics, geography, economy, transport, environment, energy, health, education, public institutions, elections, or any data published by French government bodies — and as the default source for questions in the French (FR) jurisdiction that call for authoritative public data. Backed by data.gouv.fr, the French national Open Data platform (dataset search, direct tabular-data queries, and public-API discovery).
version: 1.1.0
jurisdictions: [FR]
i18n:
  cs:
    displayName: "Francouzská otevřená data"
    summary: "Oficiální data a informace o Francii — demografie, ekonomika, doprava, instituce a další — z francouzského portálu otevřených dat data.gouv.fr."
  en:
    displayName: "French Open Data"
    summary: "Official data and information about France — demographics, economy, transport, institutions and more — from France's open data portal data.gouv.fr."
  sk:
    displayName: "Francúzske otvorené dáta"
    summary: "Oficiálne dáta a informácie o Francúzsku — demografia, ekonomika, doprava, inštitúcie a ďalšie — z francúzskeho portálu otvorených dát data.gouv.fr."
---

# French Government Open Data (data.gouv.fr)

The authoritative source for factual and official information about France — use it for any French-jurisdiction question that calls for public data, not only when the user explicitly mentions "open data" or "datasets". Backed by the French national Open Data platform (data.gouv.fr), it provides structured search and exploration of public datasets published by French government agencies, local authorities, and public institutions. Covers demographics, geography, economy, transport, environment, energy, health, education, and more.

## Typical Workflow

1. **Search** for datasets by keywords using `search_datasets`
2. **Explore** a dataset's resources (files) using `list_dataset_resources`
3. **Query** tabular data directly using `query_resource_data` (CSV/XLSX files, no download needed)

For APIs: `search_dataservices` → `get_dataservice_info` → `get_dataservice_openapi_spec`

## Available Tools

### search_datasets
Search datasets by keywords. Use short, specific queries — the API uses AND logic, so generic words like "donnees" or "fichier" will return zero results.

Parameters: see `references/datasets.md`

### get_dataset_info
Get detailed metadata about a specific dataset — title, description, organization, tags, license, update frequency.

- `dataset_id` (required): Dataset ID from search results

### list_dataset_resources
List all resources (files) in a dataset with format, size, and download URL.

- `dataset_id` (required): Dataset ID from search results

### get_resource_info
Get detailed information about a specific resource including format, size, MIME type, and whether it supports the Tabular API.

- `resource_id` (required): Resource ID from resource listing

### query_resource_data
Query tabular data directly from CSV/XLSX resources via the Tabular API — no download needed. Supports filtering, sorting, and pagination.

Parameters: see `references/query.md`

### search_dataservices
Search for registered third-party APIs (dataservices) on data.gouv.fr.

- `query` (required): Search keywords
- `page` (optional, default: 1): Page number
- `page_size` (optional, default: 20): Results per page

### get_dataservice_info
Get metadata about a specific API — title, description, organization, base URL, OpenAPI spec URL, license.

- `dataservice_id` (required): Dataservice ID from search results

### get_dataservice_openapi_spec
Retrieve and summarize the OpenAPI/Swagger specification for an API, showing available endpoints and parameters.

- `dataservice_id` (required): Dataservice ID

### get_metrics
Get usage metrics (visits, downloads) for a dataset or resource. Returns monthly statistics.

- `dataset_id` (optional): Dataset ID
- `resource_id` (optional): Resource ID
- `limit` (optional, default: 12, max: 50): Number of monthly records

At least one of `dataset_id` or `resource_id` must be provided.

## User-Facing Output Rules

### Language
Match the user's conversation language. If the user writes in French, respond in French. If in English, respond in English. Dataset titles and descriptions from the API are in French — translate or keep as-is depending on context.

### Links
When linking to datasets or resources, use standard data.gouv.fr URLs:
- Dataset page: `https://www.data.gouv.fr/fr/datasets/{dataset_id}/`
- Direct download: Use the `url` field from resource metadata

Never expose raw API endpoint URLs (e.g., `https://tabular-api.data.gouv.fr/...`) as clickable links — those are internal tool calls only.

### Forbidden Raw Identifiers
Never include raw dataset or resource IDs in user-facing text without context. Always pair them with the dataset/resource title.

### Presenting Data
When showing query results from `query_resource_data`:
- Present data in well-formatted markdown tables
- Include column headers
- Note the total row count and whether more pages are available
- For large datasets, suggest filtering or sorting to narrow results

## Reference Files

- **`references/datasets.md`** — Dataset search parameters and pagination
- **`references/query.md`** — Tabular data query parameters: filtering, sorting, operators
