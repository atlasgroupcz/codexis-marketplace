---
name: visualize-graph
description: Use when visualizing network graphs, citation networks, document relationships, or entity connections. Triggers on "graph", "network", "citations", "relationships", "connections".
---

# Graph Visualization Skill

Generate an interactive force-directed graph visualization using D3.js.

## When to Use

- Visualizing citation networks (e.g., Citex document citations)
- Showing relationships between legal entities
- Mapping document dependencies
- Displaying organizational structures with connections

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "graph",
  "title": { "literalString": "Citation Network" },
  "config": {
    "layout": "force",
    "nodeSize": 20,
    "linkDistance": 100
  },
  "data": {
    "nodes": [
      { "id": "doc1", "label": { "literalString": "Case 123/2024" }, "group": "decision" },
      { "id": "doc2", "label": { "literalString": "Law 45" }, "group": "statute" }
    ],
    "edges": [
      { "source": "doc1", "target": "doc2", "label": { "literalString": "cites" } }
    ]
  }
}
```

## Schema Fields

### config (optional)

- `layout` (string): Layout algorithm, default `"force"`
- `nodeSize` (number): Node radius in pixels, default `20`
- `linkDistance` (number): Distance between linked nodes, default `100`

### data (required)

#### nodes (required)

Array of node objects:

- `id` (string, required): Unique identifier for the node
- `label` (BoundValue): Display text for the node
- `group` (string, optional): Category for color coding (e.g., "decision", "statute", "regulation")

#### edges (required)

Array of edge objects:

- `source` (string, required): ID of the source node
- `target` (string, required): ID of the target node
- `label` (BoundValue, optional): Relationship label

## BoundValue Types

Values like `title`, `label` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "literalNumber": 42 }` - Static number
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/graph-{datetime}.html`

## Features

- Interactive drag and zoom
- Node hover tooltips
- Color-coded node groups
- Link labels on hover
- Light/dark theme support
- Schema validation with helpful error messages
