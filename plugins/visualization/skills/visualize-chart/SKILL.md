---
name: visualize-chart
description: Use when visualizing statistical data, comparisons, trends, or distributions. Triggers on "chart", "bar chart", "line chart", "pie chart", "area chart", "statistics", "comparison", "trend".
---

# Chart Visualization Skill

Generate interactive charts (bar, line, area, pie) using D3.js.

## When to Use

- Showing case statistics by category
- Comparing values across periods
- Displaying trends over time
- Visualizing proportional distributions

## A2UI Schema

```json
{
  "type": "chart",
  "chartType": "bar",
  "title": "Monthly Filings",
  "data": [
    { "month": "Jan", "civil": 120, "criminal": 45 },
    { "month": "Feb", "civil": 135, "criminal": 52 }
  ],
  "xAxis": "month",
  "series": ["civil", "criminal"]
}
```

## Schema Fields

### chartType (required)
- "bar" - Grouped or stacked bar chart
- "line" - Line chart with optional markers
- "area" - Stacked area chart
- "pie" - Pie/donut chart

### data (required)
Array of objects with:
- Category field (referenced by `xAxis`)
- Value fields (referenced by `series`)

### xAxis (required for bar/line/area)
Field name for the x-axis categories

### series (required)
Array of field names to plot as data series

### options (optional)
- `stacked` (boolean): Stack bars/areas
- `showValues` (boolean): Display values on chart
- `showLegend` (boolean): Show legend (default: true)

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-chart-{uuid}.html`

## Features

- Multiple chart types (bar, line, area, pie)
- Multi-series support with legend
- Interactive tooltips
- Animated transitions
- Light/dark theme support
