---
name: visualize
description: >-
  Create interactive data visualizations: charts (bar, line, area, pie), graphs (network, citation, relationship),
  timelines (resource schedules, Gantt charts, allocations), trees (hierarchy, structure, nested),
  calendars (scheduled events, appointments, meetings), flowcharts (workflow, process, decision),
  grids (table, data, spreadsheet), kanban (board, stages, tasks), and maps (geographic, location, coordinates).
  Triggers on "visualize", "visualization", "chart", "graph", "timeline", "Gantt", "resource schedule",
  "tree", "calendar", "schedule", "appointments", "flowchart", "workflow", "grid", "table", "kanban", "board",
  "map", "location", "diagram", "plot", "display data", "show data", "render", "draw".
---

# Visualization Router

Create interactive data visualizations for legal/law data using D3.js, AG Grid, and Leaflet.

## IMPORTANT: Read Referenced Files

This skill is a **router only**. For each visualization type, you MUST:

1. **Read the type-specific SKILL.md** for full schema documentation
2. **Read the template.html** to understand the rendering code
3. **Follow the A2UI schema format** exactly as documented

## Visualization Types

| Type | When to Use | Reference Files |
|------|-------------|-----------------|
| `chart` | Statistics, trends, comparisons (bar, line, area, pie) | [SKILL.md](references/chart/SKILL.md), [template.html](references/chart/template.html) |
| `graph` | Networks, relationships, citations | [SKILL.md](references/graph/SKILL.md), [template.html](references/graph/template.html) |
| `timeline` | Resource schedules, Gantt charts, allocations | [SKILL.md](references/timeline/SKILL.md), [template.html](references/timeline/template.html) |
| `tree` | Hierarchies, document structure, nested data | [SKILL.md](references/tree/SKILL.md), [template.html](references/tree/template.html) |
| `calendar` | Scheduled events, appointments, meetings | [SKILL.md](references/calendar/SKILL.md), [template.html](references/calendar/template.html) |
| `flowchart` | Workflows, processes, decision trees | [SKILL.md](references/flowchart/SKILL.md), [template.html](references/flowchart/template.html) |
| `grid` | Tables, search results, sortable/filterable listings | [SKILL.md](references/grid/SKILL.md), [template.html](references/grid/template.html) |
| `kanban` | Workflow stages, task boards, status columns | [SKILL.md](references/kanban/SKILL.md), [template.html](references/kanban/template.html) |
| `map` | Geographic locations, jurisdictions, markers | [SKILL.md](references/map/SKILL.md), [template.html](references/map/template.html) |

## Workflow

1. **Identify visualization type** based on user's data and intent
2. **Read the referenced SKILL.md** for that type (contains full schema with all fields)
3. **Read the template.html** for that type
4. **Construct A2UI JSON** following the schema exactly
5. **Replace placeholders** in template: `{{TITLE}}` and `{{DATA}}`
6. **Write output** to `{workDir}/{type}-{datetime}.html`

## Type Selection Guide

### Statistical Data
- Comparisons across categories → `chart` (bar)
- Trends over time → `chart` (line/area)
- Proportional distribution → `chart` (pie)

### Relationships
- Network connections, citations → `graph`
- Parent-child hierarchies → `tree`

### Temporal Data
- Resource schedules, Gantt charts → `timeline`
- Scheduled events, appointments → `calendar`

### Process/Workflow
- Decision trees, approval flows → `flowchart`
- Status-based task organization → `kanban`

### Tabular Data
- Searchable/sortable tables → `grid`

### Geographic Data
- Locations on a map → `map`

## Fallback: Custom Visualization Types

When the user requests a visualization type **not covered by the pre-defined types above**, use the dynamic fallback approach:

### When to Use Fallback

- User requests visualization types like: heatmap, treemap, sunburst, sankey, radar, chord diagram, word cloud, scatter plot matrix, parallel coordinates, etc.
- User explicitly asks for a specific library (e.g., "use Chart.js", "make it with Plotly")
- Pre-defined types don't fit the data or user's visualization goals

### Fallback Workflow

1. **Identify the best open-source library** for the requested visualization:
   - **D3.js** - Complex custom visualizations, full control
   - **Chart.js** - Simple charts with good defaults
   - **Plotly.js** - Scientific/statistical visualizations
   - **ECharts** - Rich interactive charts
   - **Vis.js** - Network graphs, timelines
   - **Cytoscape.js** - Graph theory visualizations
   - **Three.js** - 3D visualizations
   - **Vega-Lite** - Declarative grammar of graphics

2. **Follow the same A2UI schema pattern** as pre-defined types:
   ```json
   {
     "$schema": "a2ui-visualization/1.0",
     "type": "<custom-type-name>",
     "title": { "literalString": "Title" },
     "config": { /* library-specific options */ },
     "data": { /* visualization data */ }
   }
   ```

3. **Create a self-contained HTML file** that:
   - Loads the library via CDN (unpkg, cdnjs, or jsdelivr)
   - Includes light/dark theme support (matching pre-defined types)
   - Embeds the A2UI JSON data inline
   - Validates schema and shows helpful error messages
   - Is fully responsive and interactive

4. **Use CDN-hosted libraries** - Examples:
   ```html
   <!-- Chart.js -->
   <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>

   <!-- Plotly.js -->
   <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>

   <!-- ECharts -->
   <script src="https://cdn.jsdelivr.net/npm/echarts/dist/echarts.min.js"></script>

   <!-- Cytoscape.js -->
   <script src="https://cdn.jsdelivr.net/npm/cytoscape/dist/cytoscape.min.js"></script>
   ```

5. **Output path**: `{workDir}/{custom-type}-{datetime}.html`

### Fallback Template Structure

Follow the same patterns as pre-defined templates:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{TITLE}}</title>
  <!-- CDN library imports -->
  <style>
    /* Light/dark theme CSS variables */
    /* Responsive container styles */
  </style>
</head>
<body>
  <div id="visualization"></div>
  <script>
    const DATA = {{DATA}};
    // Schema validation
    // Library initialization
    // Theme detection and application
    // Responsive handling
  </script>
</body>
</html>
```

### Key Requirements for Fallback Visualizations

- **Schema validation**: Validate A2UI JSON structure before rendering
- **Error handling**: Show user-friendly error messages on invalid data
- **Theme support**: Detect system preference and support light/dark modes
- **Responsive**: Adapt to container size changes
- **Interactive**: Include tooltips, hover effects, zoom/pan where appropriate
- **Self-contained**: Single HTML file with all dependencies from CDN

## A2UI Schema Format

All visualizations use this structure:

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "<visualization-type>",
  "title": { "literalString": "Title" },
  "config": { /* type-specific rendering options */ },
  "data": { /* type-specific content data */ }
}
```

**See each type's SKILL.md for the exact `config` and `data` fields required.**

### BoundValue Pattern

String/number values can use BoundValue format:

- `{ "literalString": "Static Text" }` - Static string
- `{ "literalNumber": 42 }` - Static number
- `{ "literalBoolean": true }` - Static boolean
- `{ "path": "/data/field" }` - Data binding
- Plain strings/numbers also accepted

## Output

**Path:** `{workDir}/{type}-{datetime}.html`

## Features (All Types)

- Light/dark theme support (automatic)
- Interactive elements (zoom, pan, hover, tooltips)
- Responsive layouts
- Schema validation with helpful error messages
