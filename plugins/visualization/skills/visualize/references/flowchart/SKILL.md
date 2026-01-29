---
name: visualize-flowchart
description: Use when visualizing processes, workflows, decision trees, or step-by-step procedures. Triggers on "flowchart", "workflow", "process", "decision tree", "procedure", "steps".
---

# Flowchart Visualization Skill

Generate a directed flowchart/process diagram using D3.js.

## When to Use

- Showing legal appeal processes
- Visualizing case workflows
- Mapping decision trees
- Diagramming approval procedures

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "flowchart",
  "title": { "literalString": "Legal Appeal Process" },
  "config": {
    "direction": "TB"
  },
  "data": {
    "nodes": [
      { "id": "start", "label": { "literalString": "Start" }, "nodeType": "start" },
      { "id": "file", "label": { "literalString": "File Appeal" }, "nodeType": "process" },
      { "id": "check", "label": { "literalString": "Admissible?" }, "nodeType": "decision" },
      { "id": "review", "label": { "literalString": "Review" }, "nodeType": "process" },
      { "id": "reject", "label": { "literalString": "Rejected" }, "nodeType": "terminal" }
    ],
    "edges": [
      { "from": "start", "to": "file" },
      { "from": "file", "to": "check" },
      { "from": "check", "to": "review", "label": { "literalString": "Yes" } },
      { "from": "check", "to": "reject", "label": { "literalString": "No" } }
    ]
  }
}
```

## Schema Fields

### config (optional)

- `direction` (string): Flow direction
  - `"TB"` - Top to bottom (default)
  - `"LR"` - Left to right

### data (required)

#### nodes (required)

Array of node objects:

- `id` (string, required): Unique identifier
- `label` (BoundValue, required): Node display text
- `nodeType` (string, required): Shape type
  - `"start"` - Circle (entry point)
  - `"process"` - Rectangle (action/step)
  - `"decision"` - Diamond (conditional)
  - `"terminal"` - Rounded rectangle (end point)

#### edges (required)

Array of edge objects:

- `from` (string, required): Source node ID
- `to` (string, required): Target node ID
- `label` (BoundValue, optional): Edge label (e.g., "Yes", "No")

## BoundValue Types

Values like `title`, `label` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/flowchart-{datetime}.html`

## Features

- Multiple node shapes by type
- Directional arrows with labels
- Automatic layered layout
- Hover tooltips
- Zoom and pan support
- Light/dark theme support
- Schema validation with helpful error messages
