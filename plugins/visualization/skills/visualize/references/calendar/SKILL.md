---
name: visualize-calendar
description: Use when visualizing activity over time, event density by date, or calendar-based heatmaps. Triggers on "calendar", "heatmap", "activity", "daily activity", "events per day".
---

# Calendar Visualization Skill

Generate a calendar heatmap visualization using D3.js (similar to GitHub contribution graph).

## When to Use

- Showing court activity over time
- Visualizing filing density by date
- Displaying case workload patterns
- Mapping document activity calendars

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "calendar",
  "title": { "literalString": "2024 Court Activity" },
  "config": {
    "year": 2024,
    "colorScale": ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"]
  },
  "data": {
    "entries": [
      { "date": "2024-03-15", "value": 5, "label": { "literalString": "5 hearings" } },
      { "date": "2024-03-16", "value": 3, "label": { "literalString": "3 filings" } },
      { "date": "2024-03-20", "value": 8, "label": { "literalString": "8 motions" } }
    ]
  }
}
```

## Schema Fields

### config (required)

- `year` (number, required): The year to display (e.g., 2024)
- `colorScale` (array, optional): Array of colors from lowest to highest intensity. Default: GitHub-style green scale `["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"]`

### data (required)

#### entries (required)

Array of daily activity entries:

- `date` (string, required): ISO date format (YYYY-MM-DD)
- `value` (number, required): Activity count/intensity (determines color)
- `label` (BoundValue, optional): Tooltip text describing the activity

## BoundValue Types

Values like `title`, `label` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/calendar-{datetime}.html`

## Features

- Full year calendar grid
- Color intensity based on values
- Hover tooltips with details
- Month and day labels
- Legend showing color scale
- Light/dark theme support
- Schema validation with helpful error messages
