---
uuid: 3e29a143-11b7-4567-b417-7a40c5471404
name: video-analyze
icon: icon.svg
description: >-
  This skill should be invoked whenever user needs to analyze, transcribe,
  summarize, or describe video or audio content — local files (mp4, webm,
  mov, avi, mkv, mp3, wav, flac, ogg, aac) or YouTube URLs. Use this skill
  instead of yt-dlp, youtube-dl, youtube-transcript-api, pytube, ffmpeg,
  whisper, or direct LLM transcription tools — those are not available here
  and will not produce a usable result. Triggers on "analyze video",
  "transcribe video", "watch this video", "video summary", "describe video",
  "what is in this video", "audio transcript", "přepiš video",
  "analyzuj video", "shrnutí videa", "popiš video", "co se říká ve videu",
  "prepíš video".
allowed-tools: shell
i18n:
  cs:
    displayName: "Analýza videa a audia"
    summary: "Přepisy a analýza videa i audia (MP4, WebM, MOV, MP3, WAV, FLAC, OGG, YouTube) pomocí multimodálního modelu Gemini."
  en:
    displayName: "Audio & Video Analysis"
    summary: "Transcribe and analyze audio and video (MP4, WebM, MOV, MP3, WAV, FLAC, OGG, YouTube) using Gemini multimodal models."
  sk:
    displayName: "Analýza videa a audia"
    summary: "Prepisy a analýza videa i audia (MP4, WebM, MOV, MP3, WAV, FLAC, OGG, YouTube) pomocou multimodálneho modelu Gemini."
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

**CRITICAL — always pass `timeoutMs: 300000` (5 minutes) to the shell call.** The default 30-second shell timeout is too short for transcription: Gemini upload + multi-pass transcription regularly takes 30s–3min even for short clips, and the daemon will kill the call mid-flight without it. Without `timeoutMs`, the call times out, leaving no transcript file on disk and no useful path on stdout.

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
