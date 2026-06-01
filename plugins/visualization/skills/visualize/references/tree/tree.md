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
  "$schema": "a2ui-visualization/1.0",
  "type": "tree",
  "title": { "literalString": "Document Structure" },
  "config": {
    "orientation": "horizontal",
    "collapsible": true
  },
  "data": {
    "hierarchy": {
      "id": "root",
      "name": { "literalString": "Master Agreement" },
      "children": [
        {
          "id": "schedA",
          "name": { "literalString": "Schedule A" },
          "children": [
            { "id": "sow1", "name": { "literalString": "SOW 2024-001" } }
          ]
        },
        {
          "id": "schedB",
          "name": { "literalString": "Schedule B" },
          "value": 10
        },
        {
          "id": "amend",
          "name": { "literalString": "Amendments" },
          "children": [
            { "id": "amend1", "name": { "literalString": "Amendment 1" } }
          ]
        }
      ]
    }
  }
}
```

## Schema Fields

### config (optional)

- `orientation` (string): `"horizontal"` (default) or `"vertical"`
- `collapsible` (boolean): Enable collapse/expand on click, default `true`

### data (required)

#### hierarchy (required)

Recursive node structure:

- `id` (string, optional but recommended): Unique identifier
- `name` (BoundValue, required): Node label
- `children` (array, optional): Array of child nodes (recursive)
- `value` (number, optional): For sizing or weighting nodes
- `meta` (object, optional): Additional metadata for tooltips

## BoundValue Types

Values like `title`, `name` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/tree-{datetime}.html`

## Features

- Collapsible nodes (click to expand/collapse)
- Smooth animated transitions
- Hover tooltips with metadata
- Zoom and pan support
- Light/dark theme support
- Schema validation with helpful error messages
