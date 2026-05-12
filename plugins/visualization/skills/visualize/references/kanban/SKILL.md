---
name: visualize-kanban
description: Use when displaying workflow stages, task boards, or status-based item organization. Triggers on "kanban", "board", "workflow", "stages", "task board", "status columns".
---

# Kanban Visualization Skill

Generate a kanban board layout using CSS (not a data visualization - for workflow display).

## When to Use

- Showing change confirmation workflow stages
- Displaying task/case status boards
- Organizing items by workflow state
- Presenting approval pipelines

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "kanban",
  "title": { "literalString": "Change Confirmation Workflow" },
  "config": {
    "columnWidth": 300
  },
  "data": {
    "columns": [
      { "id": "pending", "title": { "literalString": "Pending" }, "color": "#F39C12" },
      { "id": "review", "title": { "literalString": "In Review" }, "color": "#3498DB" },
      { "id": "confirmed", "title": { "literalString": "Confirmed" }, "color": "#27AE60" }
    ],
    "cards": [
      { "id": "1", "columnId": "pending", "title": { "literalString": "CR288_2011 - Amendment" }, "subtitle": { "literalString": "3 changes" } },
      { "id": "2", "columnId": "review", "title": { "literalString": "CR145_2020 - New Section" }, "subtitle": { "literalString": "1 change" }, "tags": ["urgent"] }
    ]
  }
}
```

## Schema Fields

### config (optional)

- `columnWidth` (number): Width of each column in pixels, default `300`

### data (required)

#### columns (required)

Array of column definitions:

- `id` (string, required): Unique identifier
- `title` (BoundValue, required): Column header text
- `color` (string, optional): Accent color (hex)

#### cards (required)

Array of card objects:

- `id` (string, required): Unique identifier
- `columnId` (string, required): Column ID where card belongs
- `title` (BoundValue, required): Card title
- `subtitle` (BoundValue, optional): Secondary text
- `tags` (array, optional): Tag labels as strings
- `meta` (object, optional): Additional metadata for tooltip

## BoundValue Types

Values like `title`, `subtitle` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/kanban-{datetime}.html`

## Features

- Responsive column layout
- Card count per column header
- Tag display with colors
- Hover tooltips with metadata
- Light/dark theme support
- Schema validation with helpful error messages
