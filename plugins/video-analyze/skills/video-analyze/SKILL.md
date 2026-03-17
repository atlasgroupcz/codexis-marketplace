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

## Workflow

When the user asks you to work with a media file, **always follow this two-step approach:**

### Step 1: Generate transcript first

Before answering any question about a media file, generate a detailed transcript:

```bash
video-analyze transcript <source>
```

The binary automatically saves the transcript to `.transcripts/<filename>.transcript.json` in the current working directory (e.g. `meeting.mp4` → `.transcripts/meeting.mp4.transcript.json`, YouTube → `.transcripts/youtube-<video_id>.transcript.json`). The transcript content is also printed to stdout.

The transcript is output as a JSON array of `{ text, startSecond, endSecond }` segments, compatible with the AI SDK transcription component. Produced by Gemini Pro.

### Step 2: Answer from the transcript

Use the transcript to answer the user's question. The content is available both from stdout and from the saved file at `.transcripts/`. In most cases, the transcript contains everything you need.

### Step 3: Fall back to direct query only if needed

If the transcript does not contain enough information to answer the user's question (e.g. they ask about a very specific visual detail, a color, a logo, text on screen that wasn't captured), use the direct query mode to ask Gemini a targeted follow-up:

```bash
video-analyze <source> "<specific question about what's missing>"
```

This re-sends the media to Gemini with your specific question. Only use this when the transcript genuinely lacks the answer.

## Examples

```bash
# Step 1: Always start by generating a transcript
video-analyze transcript /home/codexis/meeting.mp4
video-analyze transcript "https://www.youtube.com/watch?v=abc123"

# Step 3: Only if transcript is insufficient, ask a targeted question
video-analyze /home/codexis/meeting.mp4 "What color is the logo shown at 2:15?"
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
