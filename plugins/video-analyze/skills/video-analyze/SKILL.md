---
name: video-analyze
description: >-
  Analyze video and audio files using Gemini multimodal models.
  Supports local files (mp4, webm, mov, avi, mkv) and YouTube URLs.
  Handles upload, authentication, and Gemini API calls automatically.
  Triggers on "analyze video", "what is in this video", "describe video",
  "video summary", "watch this video", "transcribe video".
metadata:
  skill-visibility: user
allowed-tools: shell
---

# Video & Audio Analysis

Analyze video/audio files or YouTube URLs using the `video-analyze` command.

## Usage

```bash
video-analyze <source> <query>
```

- `source` — absolute file path or YouTube URL
- `query` — what to analyze (e.g. "Transcribe this video", "Summarize the key points")

The command handles file upload, authentication, and Gemini API calls automatically.

## Examples

```bash
# Transcribe a local video
video-analyze /home/codexis/meeting.mp4 "Transcribe this video with timestamps and speaker identification"

# Summarize a YouTube video
video-analyze "https://www.youtube.com/watch?v=abc123" "Summarize the key points"

# Extract specific information
video-analyze /home/codexis/presentation.mp4 "List all products mentioned with their prices"
```

## Supported formats

- **Video:** mp4, webm, mov, avi, mkv
- **Audio:** mp3, wav, flac, ogg, aac
- **YouTube:** full URLs (youtube.com, youtu.be)

## Error handling

- **429 / rate limit errors are transient.** Wait 10–30 seconds and retry.

## Output

The command prints the Gemini response text directly to stdout.
On error, it prints an error message to stderr and exits with a non-zero code.
