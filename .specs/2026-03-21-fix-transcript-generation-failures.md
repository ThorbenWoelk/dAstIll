
## Root Cause

Two independent bugs cause transcript generation to fail:

### Bug 1: `fake_summarize.sh` (production) uses a non-existent subtitle format

The Docker image uses `scripts/fake_summarize.sh` as the `summarize` binary. This script:

1. **Requests `--sub-format ttxt`** -- YouTube never serves `ttxt` format. This first yt-dlp call always silently produces nothing (error suppressed by `|| true`).
2. **Falls through to a VTT fallback**, which is the only path that can work, but it:
   - Only checks for `transcript.en.vtt` / `transcript.vtt` -- misses non-English videos and alternate language codes
   - Only uses `--write-auto-subs` -- misses videos that only have manual subtitles
   - Has `> /dev/null 2>&1 || true` which suppresses all errors, making debugging impossible
   - Uses `set -e` with `grep` which can exit before temp dir cleanup on parse failures (leaking temp dirs)

### Bug 2: `TranscriptService::extract` makes two redundant CLI calls

The Rust service calls `summarize --extract --format md` and then `summarize --extract --format txt` sequentially. For YouTube transcripts, both produce **identical output** (I verified this). This doubles request load, doubling rate-limit exposure -- which then activates a 60-minute transcript cooldown that blocks ALL transcript processing.

## Fix Plan

### 1. Rewrite `fake_summarize.sh`

- Remove the dead `ttxt` path entirely
- Use `--sub-format vtt/srv1/srv2/srv3/best` (proper format priority list)
- Add `--write-subs` alongside `--write-auto-subs` to also get manual subtitles
- Add `--sub-langs "en.*,en"` to target English but handle variants
- Fall back to finding ANY `.vtt` file (not just `en`) if the English-specific file is absent
- Remove stdout/stderr suppression so errors propagate to the Rust backend's error detection
- Add a cleanup trap (`trap 'rm -rf "$TEMP_DIR"' EXIT`) to prevent temp dir leaks
- Improve VTT parsing to strip cue identifiers, timestamps, positioning tags, and HTML tags

### 2. Eliminate the double call in `TranscriptService::extract`

- Call `summarize --extract --format txt` once
- Use the same output for both `raw_text` and `formatted_markdown` (they are identical for YouTube)
- This halves request load and halves rate-limit exposure

### 3. Files changed

- `backend/scripts/fake_summarize.sh` -- rewritten
- `backend/src/services/transcript.rs` -- single CLI invocation instead of two
- `backend/src/services/transcript.rs` (test) -- update test to match single-call behavior
