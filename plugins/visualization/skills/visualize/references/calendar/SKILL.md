---
name: visualize-calendar
description: Use when visualizing scheduled events, appointments, meetings, or date-based activities. Triggers on "calendar", "schedule", "appointments", "meetings", "events calendar", "booking".
---

# Calendar Visualization Skill

Generate an interactive calendar visualization using Event Calendar library with month/week/day views.

## When to Use

- Showing scheduled hearings and appointments
- Visualizing meeting calendars
- Displaying case deadlines and important dates
- Mapping event schedules

## A2UI Schema

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "calendar",
  "title": { "literalString": "Court Calendar - March 2024" },
  "config": {
    "defaultView": "dayGridMonth"
  },
  "data": {
    "events": [
      {
        "id": "e1",
        "title": { "literalString": "Case Hearing #2024-001" },
        "start": "2024-03-15",
        "end": "2024-03-15",
        "allDay": true,
        "color": "#3b82f6",
        "description": { "literalString": "Initial hearing for Smith v. Jones" }
      },
      {
        "id": "e2",
        "title": { "literalString": "Filing Deadline" },
        "start": "2024-03-20",
        "allDay": true,
        "color": "#ef4444"
      },
      {
        "id": "e3",
        "title": { "literalString": "Deposition" },
        "start": "2024-03-25T09:00:00",
        "end": "2024-03-25T12:00:00",
        "allDay": false,
        "color": "#10b981"
      }
    ]
  }
}
```

### List View Example

```json
{
  "$schema": "a2ui-visualization/1.0",
  "type": "calendar",
  "title": { "literalString": "Upcoming Appointments" },
  "config": {
    "defaultView": "listWeek"
  },
  "data": {
    "events": [
      {
        "id": "e1",
        "title": { "literalString": "Team Meeting" },
        "start": "2024-03-15T10:00:00",
        "end": "2024-03-15T11:00:00",
        "allDay": false,
        "color": "#3b82f6"
      }
    ]
  }
}
```

## Schema Fields

### config (optional)

- `defaultView` (string): Initial view mode. Options:
  - **Grid Views:**
    - `"dayGridMonth"` (default) - Full month grid
    - `"dayGridWeek"` - Week grid view
    - `"dayGridDay"` - Single day grid
  - **List/Agenda Views:**
    - `"listDay"` - Single day list
    - `"listWeek"` - Week list (agenda)
    - `"listMonth"` - Month list
    - `"listYear"` - Year list

### data (required)

#### events (required)

Array of calendar events:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier |
| `title` | BoundValue | yes | Display text for the event |
| `start` | string | yes | Start date/time (ISO format: "2024-03-15" or "2024-03-15T09:00:00") |
| `end` | string | no | End date/time (ISO format). Defaults to start if omitted |
| `allDay` | boolean | no | Full-day event (default: true) |
| `color` | string | no | Background color (hex or CSS color) |
| `description` | BoundValue | no | Details shown on click |

## BoundValue Types

Values like `title`, `description` can be:

- `{ "literalString": "Static Text" }` - Static string
- `{ "path": "/data/field" }` - Data binding
- Plain strings are also accepted

## Output

Generate an HTML file using the template at `template.html` with the A2UI JSON embedded.

**Output path:** `{workDir}/calendar-{datetime}.html`

## Features

- Month, week, day, and **list/agenda** view switching
- Navigation buttons (prev/next/today)
- Click events to see descriptions
- Light/dark theme support
- View-only mode (no drag/drop editing)
- Schema validation with helpful error messages

## Breaking Changes from Previous Version

The calendar visualization has been completely rewritten:

- **Old**: GitHub-style heatmap with `config.year` and `data.entries` (date + value)
- **New**: Event Calendar with `config.defaultView` and `data.events` (scheduled events)

Old schemas using `entries` with `date`/`value` fields are no longer supported.
