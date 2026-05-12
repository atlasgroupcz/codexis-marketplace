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
  "$schema": "a2ui-visualization/1.0",
  "type": "chart",
  "title": { "literalString": "Monthly Filings" },
  "config": {
    "chartType": "bar",
    "xAxis": { "field": "month", "label": { "literalString": "Month" } },
    "series": [
      { "field": "civil", "label": { "literalString": "Civil Cases" } },
      { "field": "criminal", "label": { "literalString": "Criminal Cases" } }
    ],
    "showLegend": true,
    "stacked": false
  },
  "data": {
    "rows": [
      { "month": "Jan", "civil": 120, "criminal": 45 },
      { "month": "Feb", "civil": 135, "criminal": 52 }
    ]
  }
}
```

## Schema Fields

### config (required)

#### chartType (required)
- `"bar"` - Grouped or stacked bar chart
- `"line"` - Line chart with markers
- `"area"` - Stacked area chart
- `"pie"` - Pie/donut chart

#### xAxis (required for bar/line/area)
- `field` (string): Field name for x-axis categories
- `label` (BoundValue, optional): Axis label

#### series (required)
Array of series definitions:
- `field` (string): Field name in data rows
- `label` (BoundValue): Display label for legend

#### showLegend (optional)
Boolean, default `true`. Show/hide legend.

#### stacked (optional)
Boolean, default `false`. Stack bars/areas.

#### showValues (optional)
Boolean, default `false`. Display values on chart elements.

### data (required)

#### rows (required)
Array of objects with:
- Category field (referenced by `config.xAxis.field`)
- Value fields (referenced by `config.series[].field`)

## BoundValue Types

Values like `title`, `label` can be:
- `{ "literalString": "Static Text" }` - Static string
- `{ "literalNumber": 42 }` - Static number
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/chart-{datetime}.html`

## Features

- Multiple chart types (bar, line, area, pie)
- Multi-series support with legend
- Interactive tooltips
- Animated transitions
- Light/dark theme support
- Schema validation with helpful error messages
