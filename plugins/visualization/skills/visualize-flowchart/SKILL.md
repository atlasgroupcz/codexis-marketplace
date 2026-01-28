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
  "type": "flowchart",
  "title": "Legal Appeal Process",
  "nodes": [
    { "id": "A", "label": "File Appeal", "type": "action" },
    { "id": "B", "label": "Admissible?", "type": "decision" },
    { "id": "C", "label": "Review", "type": "action" },
    { "id": "D", "label": "Rejected", "type": "terminal" }
  ],
  "edges": [
    { "from": "A", "to": "B" },
    { "from": "B", "to": "C", "label": "Yes" },
    { "from": "B", "to": "D", "label": "No" }
  ],
  "direction": "TB"
}
```

## Schema Fields

### nodes (required)
- `id` (string): Unique identifier
- `label` (string): Node text
- `type` (string): "action" (rectangle), "decision" (diamond), "terminal" (rounded), "start" (circle)

### edges (required)
- `from` (string): Source node ID
- `to` (string): Target node ID
- `label` (string, optional): Edge label

### direction (optional)
- "TB" (top-to-bottom, default)
- "LR" (left-to-right)

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-flowchart-{uuid}.html`

## Features

- Multiple node shapes by type
- Directional arrows with labels
- Automatic layout (dagre-like)
- Zoom and pan support
- Light/dark theme support
