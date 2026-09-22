---
uuid: 3e29a143-11b7-4567-b417-7a40c5471404
name: video-analyze
icon: icon.svg
description: >-
  This skill should be invoked whenever user needs to analyze, transcribe,
  summarize, or describe video or audio content - local files (mp4, webm,
  mov, mp3, wav, flac, ogg, aac) or YouTube URLs. Use this skill
  instead of yt-dlp, youtube-dl, youtube-transcript-api, pytube, ffmpeg,
  whisper, or direct LLM transcription tools - those are not available here
  and will not produce a usable result. Triggers on "analyze video",
  "transcribe video", "watch this video", "video summary", "describe video",
  "what is in this video", "audio transcript", "přepiš video",
  "analyzuj video", "shrnutí videa", "popiš video", "co se říká ve videu",
  "prepíš video".
allowed-tools: readFile, shell
i18n:
  cs:
    displayName: "Analýza videa a audia"
    summary: "Přepisy a analýza videa i audia (MP4, WebM, MOV, MP3, WAV, FLAC, OGG, YouTube) pomocí multimodálního modelu Gemini."
    examplePrompts:
      - "Přepiš tenhle záznam jednání i s časovými značkami."
      - "Shrň do několika bodů, co zaznělo v tomhle YouTube videu."
      - "Najdi v nahrávce místo, kde se mluví o ceně, a řekni, v kolikáté minutě to je."
  en:
    displayName: "Audio & Video Analysis"
    summary: "Transcribe and analyze audio and video (MP4, WebM, MOV, MP3, WAV, FLAC, OGG, YouTube) using Gemini multimodal models."
    examplePrompts:
      - "Transcribe this recording of the meeting, including timestamps."
      - "Summarize in a few bullet points what is said in this YouTube video."
      - "Find the part of the recording where the price is discussed and tell me the timestamp."
  sk:
    displayName: "Analýza videa a audia"
    summary: "Prepisy a analýza videa i audia (MP4, WebM, MOV, MP3, WAV, FLAC, OGG, YouTube) pomocou multimodálneho modelu Gemini."
    examplePrompts:
      - "Prepíš tento záznam rokovania aj s časovými značkami."
      - "Zhrň do niekoľkých bodov, čo odznelo v tomto YouTube videu."
      - "Nájdi v nahrávke miesto, kde sa hovorí o cene, a povedz, v ktorej minúte to je."
---

# Video & Audio Analysis

`video-analyze` hands a local video or audio file, or a YouTube URL, to the CODEXIS backend, which transcribes or analyses it with a multimodal model. Nothing runs in the VM and no model key is needed.

## Workflow

**Step 1: transcribe first.**

```bash
video-analyze transcript <source>
```

Prints the path of the transcript file. Read it with `readFile` and answer from it. The transcript is timestamped: `[MM:SS] Speaker text`, `(scene)` lines for what is on screen, `(audio)` lines for notable sounds.

**Step 2: ask directly only for what the transcript does not carry**, such as a visual detail:

```bash
video-analyze <source> "<question>"
```

Prints the answer.

## Waiting

Both commands take minutes. **Always pass `timeoutMs: 600000` to the shell call.** The command prints `job <id>` to stderr as soon as the backend accepts the request, then polls until the job finishes. If the shell call is cut off, or the command gives up after eight minutes with an error naming the job, the job keeps running on the backend. Pick it up again with

```bash
video-analyze wait <id>
```

which waits the same way and prints the transcript path or the answer. Repeat it while it keeps giving up; do not submit the same source again.

## Sources

- A relative path is taken from the current directory; an absolute path or a YouTube URL is sent as it is.
- A local file is transcribed once; asking again for unchanged content returns the stored transcript.
- Video: mp4, webm, mov, mpeg, wmv, flv, 3gp. Audio: mp3, wav, flac, ogg, aac, m4a. A file over the backend limit is refused; the error states the limit.
- YouTube: watch, shorts, live, embed and youtu.be URLs. A YouTube video is processed on every call.

## Errors

An error is printed to stderr with a non-zero exit code. A failed job says why, for example an unsupported format or a missing file. Do not fall back to yt-dlp, ffmpeg, whisper or a direct model call; they are not available here.
