---
name: visualize-timeline
description: Use when visualizing temporal data, event sequences, case histories, or document version histories. Triggers on "timeline", "history", "chronology", "events over time", "temporal".
---

# Timeline Visualization Skill

Generate an interactive timeline visualization using D3.js time scales.

## When to Use

- Showing case history and proceedings
- Displaying document version history (Časová znění)
- Visualizing legislative process timelines
- Mapping event sequences with dates

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "timeline",
  "title": { "literalString": "Case History" },
  "config": {
    "orientation": "horizontal"
  },
  "data": {
    "events": [
      {
        "id": "e1",
        "label": { "literalString": "Filing" },
        "eventType": "point",
        "date": "2024-01-15"
      },
      {
        "id": "e2",
        "label": { "literalString": "Discovery Period" },
        "eventType": "range",
        "start": "2024-02-01",
        "end": "2024-04-30"
      },
      {
        "id": "e3",
        "label": { "literalString": "Decision" },
        "eventType": "point",
        "date": "2024-06-15",
        "description": { "literalString": "Final ruling issued" }
      }
    ]
  }
}
```

## Schema Fields

### config (optional)

- `orientation` (string): `"horizontal"` (default) or `"vertical"`

### data (required)

#### events (required)

Array of event objects:

- `id` (string, required): Unique identifier
- `label` (BoundValue, required): Event display text
- `eventType` (string, required): `"point"` or `"range"`
- `date` (string): ISO date for point events (e.g., "2024-01-15")
- `start` (string): ISO date for range start
- `end` (string): ISO date for range end
- `description` (BoundValue, optional): Additional details shown in tooltip

## BoundValue Types

Values like `title`, `label`, `description` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/timeline-{datetime}.html`

## Features

- Horizontal or vertical orientation
- Point events (circles) and range events (bars)
- Interactive zoom and pan
- Hover tooltips with full details
- Light/dark theme support
- Schema validation with helpful error messages
