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
  "type": "graph",
  "title": "Document Citations",
  "nodes": [
    { "id": "doc1", "label": "Case 123/2024", "group": "decision" },
    { "id": "doc2", "label": "Law §45", "group": "statute" }
  ],
  "links": [
    { "source": "doc1", "target": "doc2", "label": "cites" }
  ]
}
```

## Schema Fields

### nodes (required)
- `id` (string): Unique identifier for the node
- `label` (string): Display text for the node
- `group` (string, optional): Category for color coding (e.g., "decision", "statute", "regulation")

### links (required)
- `source` (string): ID of the source node
- `target` (string): ID of the target node
- `label` (string, optional): Relationship label

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-graph-{uuid}.html`

## Features

- Interactive drag and zoom
- Node hover tooltips
- Color-coded node groups
- Link labels on hover
- Light/dark theme support
