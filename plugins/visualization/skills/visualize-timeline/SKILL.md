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
  "type": "timeline",
  "title": "Case History",
  "events": [
    { "id": 1, "label": "Filing", "date": "2024-01-15", "type": "point" },
    { "id": 2, "label": "Discovery", "start": "2024-02-01", "end": "2024-04-30", "type": "range" },
    { "id": 3, "label": "Decision", "date": "2024-06-15", "type": "point" }
  ],
  "orientation": "horizontal"
}
```

## Schema Fields

### events (required)
- `id` (number|string): Unique identifier
- `label` (string): Event description
- `date` (string): ISO date for point events
- `start` (string): ISO date for range start
- `end` (string): ISO date for range end
- `type` (string): "point" or "range"
- `category` (string, optional): For color coding

### orientation (optional)
- "horizontal" (default) or "vertical"

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-timeline-{uuid}.html`

## Features

- Horizontal or vertical orientation
- Point events (circles) and range events (bars)
- Interactive zoom and pan
- Hover tooltips with full details
- Light/dark theme support
