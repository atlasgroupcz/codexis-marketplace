---
name: visualize-tree
description: Use when visualizing hierarchical data, document structures, organizational charts, folder hierarchies, or nested relationships. Triggers on "tree", "hierarchy", "structure", "nested", "parent-child", "folder structure".
---

# Tree Visualization Skill

Generate an interactive hierarchical tree visualization using D3.js tree layout.

## When to Use

- Showing document structure (SID folder hierarchy)
- Displaying organizational charts
- Visualizing nested legal structures
- Mapping parent-child relationships

## A2UI Schema

```json
{
  "type": "tree",
  "title": "Document Structure",
  "root": {
    "name": "Master Agreement",
    "children": [
      { "name": "Schedule A", "children": [{ "name": "SOW 2024-001" }] },
      { "name": "Schedule B" },
      { "name": "Amendments", "children": [{ "name": "Amendment 1" }] }
    ]
  }
}
```

## Schema Fields

### root (required)
- `name` (string): Node label
- `children` (array, optional): Child nodes (recursive)
- `value` (number, optional): For sizing nodes
- `meta` (object, optional): Additional metadata for tooltips

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-tree-{uuid}.html`

## Features

- Collapsible nodes (click to expand/collapse)
- Smooth animated transitions
- Hover tooltips with metadata
- Zoom and pan support
- Light/dark theme support
