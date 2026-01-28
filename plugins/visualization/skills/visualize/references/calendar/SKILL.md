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
  "type": "calendar",
  "title": "Court Activity",
  "year": 2024,
  "data": [
    { "date": "2024-03-15", "value": 5, "label": "5 hearings" },
    { "date": "2024-03-20", "value": 2, "label": "2 filings" }
  ],
  "colorScale": ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"]
}
```

## Schema Fields

### year (required)
The year to display (e.g., 2024)

### data (required)
Array of daily values:
- `date` (string): ISO date (YYYY-MM-DD)
- `value` (number): Activity count/intensity
- `label` (string, optional): Tooltip text

### colorScale (optional)
Array of colors from lowest to highest intensity.
Default: GitHub-style green scale

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-calendar-{uuid}.html`

## Features

- Full year calendar grid
- Color intensity based on values
- Hover tooltips with details
- Month labels
- Light/dark theme support
