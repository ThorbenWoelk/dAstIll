#!/usr/bin/env python3
"""Small OpenAI-compatible transcription server backed by whisper.cpp.

The server intentionally implements only the endpoint dAstIll needs:
`POST /v1/audio/transcriptions` with multipart `file`.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import socket
import subprocess
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from ipaddress import ip_address
from pathlib import Path


DEFAULT_MODEL_PATH = "/models/ggml-base.en.bin"


def env_int(name: str, default: int) -> int:
    try:
        return int(os.environ.get(name, str(default)))
    except ValueError:
        return default


ASR_API_KEY = os.environ.get("ASR_API_KEY", "").strip()
ASR_MODEL_PATH = os.environ.get("ASR_MODEL_PATH", DEFAULT_MODEL_PATH).strip()
ASR_HOST = os.environ.get("ASR_HOST", "0.0.0.0").strip()
ASR_PORT = env_int("ASR_PORT", 5092)
ASR_MAX_UPLOAD_BYTES = env_int("ASR_MAX_UPLOAD_BYTES", 300 * 1024 * 1024)
ASR_TRANSCRIBE_TIMEOUT_SECS = env_int("ASR_TRANSCRIBE_TIMEOUT_SECS", 60 * 60)
WHISPER_CLI_PATH = os.environ.get("WHISPER_CLI_PATH", "whisper-cli").strip()
FFMPEG_PATH = os.environ.get("FFMPEG_PATH", "ffmpeg").strip()


class MultipartParseError(Exception):
    pass


class AudioFetchError(Exception):
    pass


class NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def parse_content_type_boundary(header_value: str | None) -> bytes:
    if not header_value:
        raise MultipartParseError("Missing Content-Type")
    match = re.search(r"boundary=(?P<boundary>[^;]+)", header_value)
    if not match:
        raise MultipartParseError("Missing multipart boundary")
    boundary = match.group("boundary").strip().strip('"')
    if not boundary:
        raise MultipartParseError("Empty multipart boundary")
    return boundary.encode("utf-8")


def parse_content_disposition(header_block: bytes) -> dict[str, str]:
    headers = header_block.decode("utf-8", errors="replace").split("\r\n")
    disposition = next(
        (line for line in headers if line.lower().startswith("content-disposition:")),
        "",
    )
    values: dict[str, str] = {}
    for name, value in re.findall(r'([a-zA-Z0-9_-]+)="([^"]*)"', disposition):
        values[name] = value
    return values


def parse_multipart(
    body: bytes, boundary: bytes
) -> tuple[dict[str, str], dict[str, tuple[bytes, str]]]:
    fields: dict[str, str] = {}
    files: dict[str, tuple[bytes, str]] = {}
    marker = b"--" + boundary
    for raw_part in body.split(marker):
        part = raw_part
        if part.startswith(b"\r\n"):
            part = part[2:]
        if part.endswith(b"\r\n"):
            part = part[:-2]
        if not part or part == b"--":
            continue
        if part.endswith(b"--"):
            part = part[:-2].rstrip(b"\r\n")
        try:
            header_block, content = part.split(b"\r\n\r\n", 1)
        except ValueError as exc:
            raise MultipartParseError("Malformed multipart part") from exc
        disposition = parse_content_disposition(header_block)
        name = disposition.get("name")
        if not name:
            continue
        filename = disposition.get("filename")
        if filename is not None:
            files[name] = (content, filename or "audio")
        else:
            fields[name] = content.decode("utf-8", errors="replace").strip()
    return fields, files


def parse_multipart_file(body: bytes, boundary: bytes) -> tuple[bytes, str]:
    _, files = parse_multipart(body, boundary)
    if "file" not in files:
        raise MultipartParseError("Missing multipart file field")
    return files["file"]


def validate_public_url(raw_url: str) -> str:
    parsed = urllib.parse.urlparse(raw_url)
    if parsed.scheme not in {"http", "https"} or not parsed.hostname:
        raise AudioFetchError("audio_url must be an absolute HTTP(S) URL")

    try:
        addresses = socket.getaddrinfo(
            parsed.hostname, parsed.port or (443 if parsed.scheme == "https" else 80)
        )
    except socket.gaierror as exc:
        raise AudioFetchError(f"audio_url host could not be resolved: {parsed.hostname}") from exc

    for family, _, _, _, sockaddr in addresses:
        host = sockaddr[0]
        try:
            address = ip_address(host)
        except ValueError as exc:
            raise AudioFetchError(f"audio_url resolved to invalid address: {host}") from exc
        if not address.is_global:
            raise AudioFetchError("audio_url resolves to a private or local address")
    return raw_url


def filename_from_url(raw_url: str) -> str:
    path = urllib.parse.urlparse(raw_url).path
    filename = Path(path).name
    return filename or "audio"


def fetch_audio_url(raw_url: str) -> tuple[bytes, str]:
    opener = urllib.request.build_opener(NoRedirectHandler)
    current_url = raw_url
    for _ in range(10):
        validate_public_url(current_url)
        request = urllib.request.Request(current_url, headers={"User-Agent": "dAstIll-ASR/1.0"})
        try:
            response = opener.open(request, timeout=ASR_TRANSCRIBE_TIMEOUT_SECS)
        except urllib.error.HTTPError as exc:
            if exc.code in {301, 302, 303, 307, 308}:
                location = exc.headers.get("Location")
                if not location:
                    raise AudioFetchError("audio_url redirect did not include Location") from exc
                current_url = urllib.parse.urljoin(current_url, location)
                continue
            raise AudioFetchError(f"audio_url returned HTTP {exc.code}") from exc

        with response:
            length_header = response.headers.get("Content-Length")
            if length_header:
                try:
                    if int(length_header) > ASR_MAX_UPLOAD_BYTES:
                        raise AudioFetchError("audio_url content too large")
                except ValueError:
                    pass
            audio = bytearray()
            while True:
                chunk = response.read(1024 * 1024)
                if not chunk:
                    break
                audio.extend(chunk)
                if len(audio) > ASR_MAX_UPLOAD_BYTES:
                    raise AudioFetchError("audio_url content too large")
            return bytes(audio), filename_from_url(current_url)
    raise AudioFetchError("audio_url had too many redirects")


def safe_suffix(filename: str) -> str:
    suffix = Path(filename).suffix.lower()
    if suffix and re.fullmatch(r"\.[a-z0-9]{1,8}", suffix):
        return suffix
    return ".audio"


def run_transcription(audio: bytes, filename: str) -> str:
    model_path = Path(ASR_MODEL_PATH)
    if not model_path.exists():
        raise RuntimeError(f"ASR model file not found: {model_path}")
    if not shutil.which(WHISPER_CLI_PATH):
        raise RuntimeError(f"whisper-cli not found: {WHISPER_CLI_PATH}")
    if not shutil.which(FFMPEG_PATH):
        raise RuntimeError(f"ffmpeg not found: {FFMPEG_PATH}")

    with tempfile.TemporaryDirectory(prefix="dastill-asr-") as tmp:
        tmp_dir = Path(tmp)
        source_path = tmp_dir / f"input{safe_suffix(filename)}"
        wav_path = tmp_dir / "input.wav"
        source_path.write_bytes(audio)

        subprocess.run(
            [
                FFMPEG_PATH,
                "-y",
                "-loglevel",
                "error",
                "-i",
                str(source_path),
                "-ar",
                "16000",
                "-ac",
                "1",
                str(wav_path),
            ],
            check=True,
            timeout=ASR_TRANSCRIBE_TIMEOUT_SECS,
        )

        result = subprocess.run(
            [
                WHISPER_CLI_PATH,
                "-m",
                str(model_path),
                "-f",
                str(wav_path),
                "-nt",
                "-np",
                "-l",
                "en",
            ],
            check=True,
            capture_output=True,
            text=True,
            timeout=ASR_TRANSCRIBE_TIMEOUT_SECS,
        )

    return result.stdout.strip()


class Handler(BaseHTTPRequestHandler):
    server_version = "dastill-asr/1.0"

    def write_json(self, status: int, payload: dict[str, object]) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def require_auth(self) -> bool:
        if not ASR_API_KEY:
            return True
        expected = f"Bearer {ASR_API_KEY}"
        if self.headers.get("Authorization") == expected:
            return True
        self.write_json(401, {"error": "Unauthorized"})
        return False

    def do_GET(self) -> None:
        if self.path == "/health":
            self.write_json(200, {"ok": True})
            return
        if self.path == "/v1/models":
            self.write_json(200, {"data": [{"id": Path(ASR_MODEL_PATH).stem}]})
            return
        self.write_json(404, {"error": "Not found"})

    def do_POST(self) -> None:
        if self.path != "/v1/audio/transcriptions":
            self.write_json(404, {"error": "Not found"})
            return
        if not self.require_auth():
            return

        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            self.write_json(400, {"error": "Invalid Content-Length"})
            return
        if length <= 0:
            self.write_json(400, {"error": "Missing request body"})
            return
        if length > ASR_MAX_UPLOAD_BYTES:
            self.write_json(413, {"error": "Audio upload too large"})
            return

        try:
            body = self.rfile.read(length)
            boundary = parse_content_type_boundary(self.headers.get("Content-Type"))
            fields, files = parse_multipart(body, boundary)
            if "file" in files:
                audio, filename = files["file"]
            elif fields.get("audio_url"):
                audio, filename = fetch_audio_url(fields["audio_url"])
            else:
                raise MultipartParseError("Missing multipart file field or audio_url field")
            transcript = run_transcription(audio, filename)
        except MultipartParseError as exc:
            self.write_json(400, {"error": str(exc)})
            return
        except AudioFetchError as exc:
            self.write_json(400, {"error": str(exc)})
            return
        except subprocess.TimeoutExpired:
            self.write_json(503, {"error": "Transcription timed out"})
            return
        except subprocess.CalledProcessError as exc:
            self.write_json(500, {"error": f"Transcription command failed: {exc}"})
            return
        except Exception as exc:  # noqa: BLE001 - surface as JSON for Cloud Run logs/users
            self.write_json(500, {"error": str(exc)})
            return

        self.write_json(200, {"text": transcript})

    def log_message(self, fmt: str, *args: object) -> None:
        print(f"{self.address_string()} - {fmt % args}", flush=True)


def main() -> None:
    print(
        f"Starting dAstIll ASR on {ASR_HOST}:{ASR_PORT} with model {ASR_MODEL_PATH}",
        flush=True,
    )
    ThreadingHTTPServer((ASR_HOST, ASR_PORT), Handler).serve_forever()


if __name__ == "__main__":
    main()
