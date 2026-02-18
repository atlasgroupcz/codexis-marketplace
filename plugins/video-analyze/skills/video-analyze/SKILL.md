---
name: video-analyze
description: >-
  Analyze video content using Gemini. Supports YouTube URLs and local video files.
  Supported formats: mp4, webm, mov, mpeg, flv, wmv, avi, 3gpp.
  Other formats are converted automatically by the command.
  Triggers on "analyze video", "what is in this video", "describe video",
  "video summary", "watch this video".
metadata:
  skill-visibility: model
allowed-tools: shell
---

# Video Analysis

Analyze video content using the `video-analyze` command available in the shell.

## Usage

```bash
video-analyze <source> <query>
```

- `source` — YouTube URL or absolute file path to video
- `query` — What to analyze in the video (be specific)

## Supported Formats

- **Direct upload:** mp4, webm, mov, mpeg, flv, wmv, avi, 3gpp
- **Any other format:** `video-analyze` converts it to mp4 automatically (no manual conversion needed)
- **YouTube:** full URLs (youtube.com, youtu.be)

## When to Use

- User asks to analyze, describe, or summarize a video
- User asks about the content of a video file or YouTube link
- User wants to extract information from video content

## When NOT to Use

- Audio-only files — this is for video content
- Image files — use vision/image analysis instead

## How It Works

1. Run `video-analyze <source> "<query>"`
2. The command returns JSON: `{"response": "analysis text..."}`
3. Present the analysis to the user

## Examples

```bash
# Analyze a YouTube video
video-analyze "https://www.youtube.com/watch?v=abc123" "What is this video about?"

# Analyze a local video file
video-analyze /home/codexis/recording.mp4 "Describe the main topics discussed"

# Extract specific information
video-analyze /home/codexis/presentation.mp4 "List all products mentioned with their prices"
```

## Error Handling

On error, the command returns JSON with an error message:
```json
{"error": "Failed to analyze video: ..."}
```

## Constraints

- Requires GEMINI_API_KEY set in ~/.cdx/.env
