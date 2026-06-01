---
name: visualize-grid
description: Use when displaying tabular data with sorting, filtering, or pagination. Triggers on "table", "grid", "data table", "spreadsheet", "list view", "search results".
---

# Grid Visualization Skill

Generate an interactive data grid using AG Grid (not a visualization - for tabular data display).

## When to Use

- Displaying search results
- Showing case listings
- Presenting document metadata
- Any tabular data with sorting/filtering needs

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "grid",
  "title": { "literalString": "Case Search Results" },
  "config": {
    "columns": [
      { "field": "caseNumber", "header": { "literalString": "Case #" }, "sortable": true, "filter": true },
      { "field": "parties", "header": { "literalString": "Parties" } },
      { "field": "court", "header": { "literalString": "Court" }, "filter": true },
      { "field": "status", "header": { "literalString": "Status" } }
    ],
    "pagination": true,
    "pageSize": 25
  },
  "data": {
    "rows": [
      { "caseNumber": "2024-CV-1234", "parties": "Smith v. Jones", "court": "District", "status": "Active" },
      { "caseNumber": "2024-CV-5678", "parties": "Brown v. Davis", "court": "Appeals", "status": "Pending" }
    ]
  }
}
```

## Schema Fields

### config (required)

#### columns (required)

Array of column definitions:

- `field` (string, required): Data field name matching row properties
- `header` (BoundValue, optional): Column header text
- `sortable` (boolean, optional): Enable sorting, default `false`
- `filter` (boolean, optional): Enable filtering, default `false`
- `width` (number, optional): Column width in pixels

#### pagination (optional)

- `pagination` (boolean): Enable pagination, default `false`
- `pageSize` (number): Rows per page, default `25`

### data (required)

#### rows (required)

Array of row objects with properties matching column `field` names.

## BoundValue Types

Values like `title`, `header` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/grid-{datetime}.html`

## Features

- Column sorting (click headers)
- Column filtering (filter inputs)
- Pagination with configurable page size
- Row selection
- Light/dark theme support (ag-theme-alpine)
- Schema validation with helpful error messages
