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
  "type": "map",
  "title": "Court Jurisdictions",
  "center": [50.0755, 14.4378],
  "zoom": 7,
  "markers": [
    { "lat": 50.0755, "lng": 14.4378, "label": "Supreme Court", "popup": "Praha" },
    { "lat": 49.1951, "lng": 16.6068, "label": "Regional Court", "popup": "Brno" }
  ]
}
```

## Schema Fields

### center (required)
Array of [latitude, longitude] for initial map center

### zoom (optional)
Initial zoom level (1-18, default: 10)

### markers (optional)
Array of map markers:
- `lat` (number): Latitude
- `lng` (number): Longitude
- `label` (string): Marker label
- `popup` (string, optional): Popup content on click
- `color` (string, optional): Marker color

### bounds (optional)
Array of [[lat1, lng1], [lat2, lng2]] to fit map to bounds

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `~/.cdx/chats/{sanitized-workdir}/{chatId}/visualization-map-{uuid}.html`

## Features

- Interactive pan and zoom
- Marker clusters (for many points)
- Click popups
- Multiple tile providers
- Light/dark theme support
