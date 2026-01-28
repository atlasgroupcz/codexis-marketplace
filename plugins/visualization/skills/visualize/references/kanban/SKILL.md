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
  "type": "kanban",
  "title": "Change Confirmation Workflow",
  "columns": [
    { "id": "pending", "title": "Pending", "color": "#F39C12" },
    { "id": "review", "title": "In Review", "color": "#3498DB" },
    { "id": "confirmed", "title": "Confirmed", "color": "#27AE60" }
  ],
  "cards": [
    { "id": 1, "column": "pending", "title": "CR288_2011 - Amendment", "subtitle": "3 changes" },
    { "id": 2, "column": "review", "title": "CR145_2020 - New Section", "subtitle": "1 change", "tags": ["urgent"] }
  ]
}
```

## Schema Fields

### columns (required)
Array of column definitions:
- `id` (string): Unique identifier
- `title` (string): Column header text
- `color` (string, optional): Accent color (hex)

### cards (required)
Array of card objects:
- `id` (number|string): Unique identifier
- `column` (string): Column ID where card belongs
- `title` (string): Card title
- `subtitle` (string, optional): Secondary text
- `tags` (array, optional): Tag labels
- `meta` (object, optional): Additional metadata for tooltip

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-kanban-{uuid}.html`

## Features

- Responsive column layout
- Card count per column
- Tag display with colors
- Hover tooltips with metadata
- Light/dark theme support
