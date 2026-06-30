# TTS

dAstIll uses Google Cloud Text-to-Speech to synthesize summary audio.

Configuration keys live in `backend/.env.example`.

| Setting       | Purpose               |
| ------------- | --------------------- |
| Enabled flag  | Enables TTS           |
| Voice         | Google TTS voice name |
| Language      | Google TTS language   |
| Output format | Audio output format   |
| Sample rate   | Sample rate in Hz     |

Pipeline:

1. Summary markdown is sanitized.
2. SSML pause markers are inserted after headings and list items.
3. Long text is split under 2500 characters while preserving SSML tag boundaries.
4. Chunks are sent to Google Cloud Text-to-Speech.
5. Returned PCM audio is concatenated.
6. PCM is wrapped as WAV for browser playback.

Generated audio cache ownership and `tts_stats` storage live in
[Data Model](/architecture/data-model#generated-audio-cache).
