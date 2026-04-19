#!/bin/zsh
set -euo pipefail

mode="attached"
case "${1:-}" in
	"")
		;;
	--detach)
		mode="detach"
		;;
	*)
		echo "Usage: ./scripts/start_local_asr.sh [--detach]"
		exit 1
		;;
esac

asr_port=${LOCAL_ASR_PORT:-5092}
asr_host=${LOCAL_ASR_HOST:-127.0.0.1}
model_path=${LOCAL_ASR_MODEL_PATH:-${HOME}/.cache/dastill/asr/ggml-base.en.bin}
model_url=${LOCAL_ASR_MODEL_URL:-https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin}
repo_root=${0:A:h:h}
pid_file="${repo_root}/.local-asr.pid"
log_file="${repo_root}/local-asr.log"

if ! command -v whisper-server >/dev/null 2>&1; then
	echo "whisper-server not found. Install the maintained whisper.cpp package first:"
	echo "  brew install whisper-cpp"
	exit 1
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
	echo "ffmpeg not found. Install it first:"
	echo "  brew install ffmpeg"
	exit 1
fi

if [[ ! -f "$model_path" ]]; then
	echo "Downloading whisper.cpp model to $model_path"
	mkdir -p "${model_path:h}"
	curl -L --fail --retry 3 -o "$model_path" "$model_url"
fi

if lsof -nP -iTCP:"$asr_port" -sTCP:LISTEN >/dev/null 2>&1; then
	echo "Local ASR already listening on ${asr_host}:${asr_port}"
	exit 0
fi

cmd=(
	whisper-server
	--host "$asr_host"
	--port "$asr_port"
	--inference-path /v1/audio/transcriptions
	--convert
	--model "$model_path"
	--no-timestamps
)

if [[ "$mode" == "detach" ]]; then
	echo "Starting local ASR on http://${asr_host}:${asr_port}/v1/audio/transcriptions (log: local-asr.log)"
	"${cmd[@]}" >"$log_file" 2>&1 &
	echo $! >"$pid_file"
else
	echo "Starting local ASR on http://${asr_host}:${asr_port}/v1/audio/transcriptions"
	"${cmd[@]}"
fi
