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
  "type": "grid",
  "title": "Case Search Results",
  "columnDefs": [
    { "field": "caseNumber", "headerName": "Case #", "sortable": true, "filter": true },
    { "field": "parties", "headerName": "Parties" },
    { "field": "court", "headerName": "Court", "filter": true },
    { "field": "status", "headerName": "Status" }
  ],
  "rowData": [
    { "caseNumber": "2024-CV-1234", "parties": "Smith v. Jones", "court": "District", "status": "Active" }
  ]
}
```

## Schema Fields

### columnDefs (required)
Array of column definitions:
- `field` (string): Data field name
- `headerName` (string, optional): Column header text
- `sortable` (boolean, optional): Enable sorting
- `filter` (boolean, optional): Enable filtering
- `width` (number, optional): Column width in pixels

### rowData (required)
Array of row objects matching column field names

### options (optional)
- `pagination` (boolean): Enable pagination
- `pageSize` (number): Rows per page (default: 25)

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-grid-{uuid}.html`

## Features

- Column sorting
- Column filtering
- Pagination
- Row selection
- Light/dark theme support (ag-theme-alpine)
