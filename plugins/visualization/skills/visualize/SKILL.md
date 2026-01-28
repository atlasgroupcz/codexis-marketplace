---
name: visualize
description: >-
  Create interactive data visualizations: charts (bar, line, area, pie), graphs (network, citation, relationship),
  timelines (chronology, history, events), trees (hierarchy, structure, nested), calendars (heatmap, activity, daily),
  flowcharts (workflow, process, decision), grids (table, data, spreadsheet), kanban (board, stages, tasks),
  and maps (geographic, location, coordinates). Triggers on "visualize", "visualization", "chart", "graph",
  "timeline", "tree", "calendar", "heatmap", "flowchart", "workflow", "grid", "table", "kanban", "board",
  "map", "location", "diagram", "plot", "display data", "show data", "render", "draw".
---

# Visualization Router

Create interactive data visualizations for legal/law data using D3.js, AG Grid, and Leaflet.

This is the main entry point for all visualization types. Based on your data and intent, select the appropriate visualization type below.

## Quick Reference

| Type | Use Case | Key Fields | Detailed Skill |
|------|----------|------------|----------------|
| `chart` | Statistics, trends, comparisons | `chartType`, `data`, `xAxis`, `series` | [chart](references/chart/SKILL.md) |
| `graph` | Networks, relationships, citations | `nodes`, `links` | [graph](references/graph/SKILL.md) |
| `timeline` | Event sequences, case history | `events`, `orientation` | [timeline](references/timeline/SKILL.md) |
| `tree` | Hierarchies, document structure | `root` (with `children`) | [tree](references/tree/SKILL.md) |
| `calendar` | Activity heatmaps, daily data | `year`, `data` | [calendar](references/calendar/SKILL.md) |
| `flowchart` | Workflows, processes, decisions | `nodes`, `edges`, `direction` | [flowchart](references/flowchart/SKILL.md) |
| `grid` | Tables, search results, listings | `columnDefs`, `rowData` | [grid](references/grid/SKILL.md) |
| `kanban` | Workflow stages, task boards | `columns`, `cards` | [kanban](references/kanban/SKILL.md) |
| `map` | Geographic locations, jurisdictions | `center`, `markers` | [map](references/map/SKILL.md) |

## How to Use

1. **Determine visualization type** based on data structure and user intent
2. **Read the detailed skill** for full schema documentation (links above)
3. **Construct A2UI JSON** with `type` field set to chosen type
4. **Read the template** from `references/{type}/template.html`
5. **Replace placeholders** `{{TITLE}}` and `{{DATA}}`
6. **Write output** to `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-{type}-{uuid}.html`

## Type Selection Guide

### For Statistical Data
- **Comparisons across categories** → `chart` (bar)
- **Trends over time** → `chart` (line/area)
- **Proportional distribution** → `chart` (pie)

### For Relationships
- **Network connections** → `graph`
- **Citation networks** → `graph`
- **Parent-child hierarchies** → `tree`

### For Temporal Data
- **Event sequences** → `timeline`
- **Activity density by day** → `calendar`

### For Process/Workflow
- **Decision trees** → `flowchart`
- **Status-based organization** → `kanban`

### For Tabular Data
- **Searchable/sortable tables** → `grid`

### For Geographic Data
- **Locations on map** → `map`

---

## Visualization Type Details

### Chart (`type: "chart"`)

**Library:** D3.js | **Skill:** [chart](references/chart/SKILL.md)

Interactive charts for statistics, trends, and comparisons.

**Subtypes:** `bar`, `line`, `area`, `pie`

```json
{
  "type": "chart",
  "chartType": "bar",
  "title": "Monthly Filings",
  "data": [{ "month": "Jan", "civil": 120, "criminal": 45 }],
  "xAxis": "month",
  "series": ["civil", "criminal"]
}
```

---

### Graph (`type: "graph"`)

**Library:** D3.js | **Skill:** [graph](references/graph/SKILL.md)

Force-directed network graphs for relationships and citations.

```json
{
  "type": "graph",
  "title": "Document Citations",
  "nodes": [{ "id": "doc1", "label": "Case 123/2024", "group": "decision" }],
  "links": [{ "source": "doc1", "target": "doc2", "label": "cites" }]
}
```

---

### Timeline (`type: "timeline"`)

**Library:** D3.js | **Skill:** [timeline](references/timeline/SKILL.md)

Temporal event sequences with point and range events.

```json
{
  "type": "timeline",
  "title": "Case History",
  "events": [
    { "id": "e1", "label": "Filing", "type": "point", "date": "2024-01-15" },
    { "id": "e2", "label": "Discovery", "type": "range", "start": "2024-02-01", "end": "2024-03-15" }
  ],
  "orientation": "horizontal"
}
```

---

### Tree (`type: "tree"`)

**Library:** D3.js | **Skill:** [tree](references/tree/SKILL.md)

Collapsible hierarchical tree for nested structures.

```json
{
  "type": "tree",
  "title": "Document Structure",
  "root": {
    "name": "Root",
    "children": [
      { "name": "Child 1", "value": 10 },
      { "name": "Child 2", "children": [{ "name": "Grandchild" }] }
    ]
  }
}
```

---

### Calendar (`type: "calendar"`)

**Library:** D3.js | **Skill:** [calendar](references/calendar/SKILL.md)

Calendar heatmap showing activity density by day.

```json
{
  "type": "calendar",
  "title": "2024 Filing Activity",
  "year": 2024,
  "data": [
    { "date": "2024-01-15", "value": 5, "label": "5 filings" },
    { "date": "2024-01-16", "value": 12, "label": "12 filings" }
  ]
}
```

---

### Flowchart (`type: "flowchart"`)

**Library:** D3.js | **Skill:** [flowchart](references/flowchart/SKILL.md)

Directed process diagrams and decision trees.

```json
{
  "type": "flowchart",
  "title": "Appeal Process",
  "nodes": [
    { "id": "start", "label": "File Appeal", "type": "start" },
    { "id": "review", "label": "Review?", "type": "decision" }
  ],
  "edges": [{ "from": "start", "to": "review" }],
  "direction": "TB"
}
```

---

### Grid (`type: "grid"`)

**Library:** AG Grid | **Skill:** [grid](references/grid/SKILL.md)

Interactive data tables with sorting, filtering, pagination.

```json
{
  "type": "grid",
  "title": "Search Results",
  "columnDefs": [
    { "field": "caseNo", "headerName": "Case Number", "sortable": true, "filter": true }
  ],
  "rowData": [{ "caseNo": "123/2024", "date": "2024-01-15" }],
  "options": { "pagination": true, "pageSize": 25 }
}
```

---

### Kanban (`type: "kanban"`)

**Library:** CSS | **Skill:** [kanban](references/kanban/SKILL.md)

Workflow boards with columns and cards.

```json
{
  "type": "kanban",
  "title": "Case Workflow",
  "columns": [
    { "id": "todo", "title": "To Do", "color": "#f59e0b" },
    { "id": "done", "title": "Done", "color": "#10b981" }
  ],
  "cards": [{ "id": "1", "column": "todo", "title": "Case 123", "tags": ["urgent"] }]
}
```

---

### Map (`type: "map"`)

**Library:** Leaflet | **Skill:** [map](references/map/SKILL.md)

Interactive tile maps with markers and popups.

```json
{
  "type": "map",
  "title": "Court Locations",
  "center": [50.0755, 14.4378],
  "zoom": 10,
  "markers": [
    { "lat": 50.0755, "lng": 14.4378, "label": "Prague Court", "popup": "District Court" }
  ]
}
```

---

## Template Paths

| Type | Template Location |
|------|-------------------|
| `calendar` | `references/calendar/template.html` |
| `chart` | `references/chart/template.html` |
| `flowchart` | `references/flowchart/template.html` |
| `graph` | `references/graph/template.html` |
| `grid` | `references/grid/template.html` |
| `kanban` | `references/kanban/template.html` |
| `map` | `references/map/template.html` |
| `timeline` | `references/timeline/template.html` |
| `tree` | `references/tree/template.html` |

## Features (All Types)

- Light/dark theme support (automatic)
- Interactive elements (zoom, pan, hover, tooltips)
- Responsive layouts
- Consistent visual styling via Tailwind CSS

## Output

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-{type}-{uuid}.html`
