---
name: visualize-map
description: Use when displaying geographic locations, court jurisdictions, or location-based data on a tile map. Triggers on "map", "location", "geographic", "jurisdiction", "coordinates".
---

# Map Visualization Skill

Generate an interactive tile map using Leaflet (D3 cannot render tile maps).

## When to Use

- Showing court locations and jurisdictions
- Displaying geographic case data
- Mapping office/branch locations
- Visualizing regional statistics

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "map",
  "title": { "literalString": "Court Jurisdictions" },
  "config": {
    "center": [50.0755, 14.4378],
    "zoom": 7
  },
  "data": {
    "markers": [
      {
        "id": "m1",
        "coordinates": { "lat": 50.0755, "lng": 14.4378 },
        "label": { "literalString": "Supreme Court" },
        "popup": { "literalString": "Praha - Nejvyšší soud" }
      },
      {
        "id": "m2",
        "coordinates": { "lat": 49.1951, "lng": 16.6068 },
        "label": { "literalString": "Regional Court" },
        "popup": { "literalString": "Brno - Krajský soud" },
        "color": "#3498DB"
      }
    ]
  }
}
```

## Schema Fields

### config (required)

- `center` (array, required): Initial map center as `[latitude, longitude]`
- `zoom` (number, optional): Initial zoom level (1-18), default `10`
- `bounds` (array, optional): Fit map to bounds `[[lat1, lng1], [lat2, lng2]]`

### data (required)

#### markers (optional)

Array of map markers:

- `id` (string, required): Unique identifier
- `coordinates` (object, required): Position as `{ "lat": number, "lng": number }`
- `label` (BoundValue, required): Marker label/title
- `popup` (BoundValue, optional): Popup content on click
- `color` (string, optional): Marker color (hex)

## BoundValue Types

Values like `title`, `label`, `popup` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/map-{datetime}.html`

## Features

- Interactive pan and zoom
- Marker clusters (for many points)
- Click popups with content
- Multiple tile providers
- Light/dark theme support
- Schema validation with helpful error messages
