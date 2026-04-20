---
uuid: 3e29a143-11b7-4567-b417-7a40c5471404
name: video-analyze
description: >-
  Analyze video and audio files using Gemini multimodal models.
  Supports local files (mp4, webm, mov, avi, mkv) and YouTube URLs.
  Handles upload, authentication, and Gemini API calls automatically.
  Triggers on "analyze video", "what is in this video", "describe video",
  "video summary", "watch this video", "transcribe video".
allowed-tools: shell
i18n:
  cs:
    displayName: "Analýza videí"
    summary: "Analýza obsahu videí (MP4, YouTube a další) pomocí multimodálního modelu Gemini."
  en:
    displayName: "Video Analysis"
    summary: "Analyze video content (MP4, YouTube and more) using Gemini multimodal models."
  sk:
    displayName: "Analýza videí"
    summary: "Analýza obsahu videí (MP4, YouTube a ďalšie) pomocou multimodálneho modelu Gemini."
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

The command prints the **file path** to stdout (e.g. `.transcripts/meeting.mp4.transcript.txt`). The transcript itself is saved to that file — read it with `readFile` to access the content.

The transcript uses a simple timestamped format with spoken dialogue, scene descriptions, and audio cues. Produced by Gemini Pro.

**Important:** Use a generous shell timeout (e.g. `timeoutMs: 300000`) — transcription of long media can take several minutes.

### Step 2: Answer from the transcript

Read the transcript file and use its contents to answer the user's question. In most cases, the transcript contains everything you need.

### Step 3: Fall back to direct query only if needed

If the transcript does not contain enough information to answer the user's question (e.g. they ask about a very specific visual detail not captured in the transcript), use the direct query mode:

```bash
video-analyze <source> "<specific question about what's missing>"
```

This re-sends the media to Gemini with your specific question. Only use this when the transcript genuinely lacks the answer.

## Examples

```bash
# Step 1: Generate transcript (prints file path)
video-analyze transcript /home/codexis/meeting.mp4
# → .transcripts/meeting.mp4.transcript.txt

# Then read the transcript
readFile .transcripts/meeting.mp4.transcript.txt

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

- `transcript` mode: prints the saved file path to stdout
- Query mode: prints the Gemini response text directly to stdout
- On error, prints an error message to stderr and exits with a non-zero code.
