---
name: visualize-timeline
description: Use when visualizing resource schedules, Gantt charts, project timelines, or resource allocations. Triggers on "timeline", "Gantt", "resource schedule", "project plan", "allocation", "resource timeline".
---

# Timeline Visualization Skill

Generate an interactive resource timeline visualization using Event Calendar library with ResourceTimeline views.

## When to Use

- Showing resource allocation over time
- Visualizing project Gantt charts
- Displaying courtroom or judge schedules
- Mapping team assignments and workloads

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "timeline",
  "title": { "literalString": "Project Timeline Q1 2024" },
  "config": {
    "defaultView": "resourceTimelineMonth",
    "slotDuration": "1 day"
  },
  "data": {
    "resources": [
      {
        "id": "r1",
        "title": { "literalString": "Courtroom A" },
        "color": "#3b82f6"
      },
      {
        "id": "r2",
        "title": { "literalString": "Courtroom B" },
        "color": "#10b981"
      }
    ],
    "events": [
      {
        "id": "e1",
        "resourceId": "r1",
        "title": { "literalString": "Smith v. Jones Trial" },
        "start": "2024-01-05",
        "end": "2024-01-20",
        "color": "#3b82f6",
        "description": { "literalString": "Civil case #2024-001" }
      },
      {
        "id": "e2",
        "resourceId": "r1",
        "title": { "literalString": "Maintenance" },
        "start": "2024-01-22",
        "end": "2024-01-24",
        "color": "#f59e0b"
      },
      {
        "id": "e3",
        "resourceId": "r2",
        "title": { "literalString": "State v. Williams" },
        "start": "2024-01-08",
        "end": "2024-01-15",
        "color": "#ef4444"
      },
      {
        "id": "e4",
        "resourceId": "r2",
        "title": { "literalString": "Johnson Hearing" },
        "start": "2024-01-18",
        "end": "2024-01-19"
      }
    ]
  }
}
```

## Schema Fields

### config (optional)

- `defaultView` (string): Initial view mode. Options:
  - `"resourceTimelineMonth"` (default) - Month view with resources
  - `"resourceTimelineWeek"` - Week view with resources
  - `"resourceTimelineDay"` - Day view with resources
- `slotDuration` (string): Time slot granularity. Examples:
  - `"1 day"` (default)
  - `"1 hour"`
  - `"30 minutes"`
  - `"1 week"`

### data (required)

#### resources (required)

Array of resources (rows in the timeline):

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier |
| `title` | BoundValue | yes | Display name for the resource |
| `color` | string | no | Default event color for this resource |

#### events (required)

Array of timeline events:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier |
| `resourceId` | string | yes | ID of the resource this event belongs to |
| `title` | BoundValue | yes | Display text for the event |
| `start` | string | yes | Start date (ISO format: "2024-01-05") |
| `end` | string | yes | End date (ISO format: "2024-01-20") |
| `color` | string | no | Background color (overrides resource color) |
| `description` | BoundValue | no | Details shown on click |

## BoundValue Types

Values like `title`, `description` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/timeline-{datetime}.html`

## Features

- Resource-based timeline (Gantt-style)
- Month, week, and day view switching
- Configurable time slot duration
- Navigation buttons (prev/next/today)
- Click events to see descriptions
- Light/dark theme support
- View-only mode (no drag/drop editing)
- Schema validation with helpful error messages

## Breaking Changes from Previous Version

The timeline visualization has been completely rewritten:

- **Old**: D3.js timeline with `eventType` (point/range), `date`, `start`, `end` fields
- **New**: Event Calendar ResourceTimeline with explicit `resources` and `events` arrays

Old schemas using `eventType: "point"` or flat events without `resourceId` are no longer supported.
All events must now be assigned to a resource.
