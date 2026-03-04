#!/bin/bash
# A wrapper around yt-dlp to provide a compatible interface for TranscriptService

set -e

VIDEO_URL=""
FORMAT="txt"

# Simple parser for the expected arguments: <URL> --extract --format <md|txt>
while [[ "$#" -gt 0 ]]; do
    case $1 in
        https*) VIDEO_URL="$1" ;;
        --format) shift; FORMAT="$1" ;;
        --extract) ;; # ignored
        *) ;;
    esac
    shift
done

if [ -z "$VIDEO_URL" ]; then
    echo "Usage: summarize <URL> --extract --format <md|txt>" >&2
    exit 1
fi

# Use yt-dlp to get the transcript
# --get-subs --skip-download --sub-format "vtt/srv1/srv2/srv3/ttml"
# We'll use --write-auto-subs to get automated transcripts if manual ones are missing
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

yt-dlp --write-auto-subs --skip-download --sub-format ttxt --output "transcript" "$VIDEO_URL" > /dev/null 2>&1 || true

# Check if we got something
if [ -f "transcript.en.ttxt" ]; then
    # Simple extraction from ttxt (which is XML-like)
    # This is a very crude parser but might work for simple text
    grep -oP '(?<=<p[^>]*>).*?(?=</p>)' transcript.en.ttxt | sed 's/<[^>]*>//g' | sed 's/&quot;/"/g' | sed 's/&#39;/'"'"'/g'
elif [ -f "transcript.ttxt" ]; then
    grep -oP '(?<=<p[^>]*>).*?(?=</p>)' transcript.ttxt | sed 's/<[^>]*>//g' | sed 's/&quot;/"/g' | sed 's/&#39;/'"'"'/g'
else
    # Try another format if ttxt failed
    yt-dlp --write-auto-subs --skip-download --sub-format vtt --output "transcript" "$VIDEO_URL" > /dev/null 2>&1 || true
    if [ -f "transcript.en.vtt" ]; then
        grep -vE '^[0-9]{2}:[0-9]{2}:[0-9]{2}' transcript.en.vtt | grep -vE '^WEBVTT|^Kind:|^Language:' | sed '/^[[:space:]]*$/d'
    elif [ -f "transcript.vtt" ]; then
        grep -vE '^[0-9]{2}:[0-9]{2}:[0-9]{2}' transcript.vtt | grep -vE '^WEBVTT|^Kind:|^Language:' | sed '/^[[:space:]]*$/d'
    else
        echo "No transcript available" >&2
        exit 1
    fi
fi

rm -rf "$TEMP_DIR"
