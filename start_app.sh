#!/bin/zsh
set -euo pipefail

mode="attached"
case "${1:-}" in
	"")
		;;
	--detach)
		mode="detach"
		;;
	--detached-child)
		mode="detached_child"
		;;
	*)
		echo "Usage: ./start_app.sh [--detach]"
		exit 1
		;;
esac

frontend_port=${FRONTEND_PORT:-3543}
backend_port=${BACKEND_PORT:-3544}
docs_port=${DOCS_PORT:-4173}
ports=($frontend_port $backend_port $docs_port)
script_path=${0:A}

wait_for_http() {
	local name=$1
	local url=$2
	local max_retries=${3:-30}
	local attempt=1

	while (( attempt <= max_retries )); do
		if curl -fsS "$url" >/dev/null 2>&1; then
			return 0
		fi
		sleep 1
		((attempt++))
	done

	echo "$name did not become ready at $url"
	return 1
}

start_backend() {
	pushd backend >/dev/null
	PORT=$backend_port cargo run --bin dastill > >(tee ../backend.log) 2>&1 &
	backend_pid=$!
	popd >/dev/null
}

start_frontend() {
	pushd frontend >/dev/null
	VITE_API_BASE="http://localhost:$backend_port" \
		bun run dev -- --host 0.0.0.0 --port $frontend_port > >(tee ../frontend.log) 2>&1 &
	frontend_pid=$!
	popd >/dev/null
}

start_docs() {
	pushd docs >/dev/null
	./node_modules/.bin/vitepress dev . --host 0.0.0.0 --port $docs_port > >(tee ../docs.log) 2>&1 &
	docs_pid=$!
	popd >/dev/null
}

cleanup() {
	set +e
	for port in "${ports[@]}"; do
		pids=$(lsof -ti :"$port" 2>/dev/null)
		if [[ -n "$pids" ]]; then
			echo "Killing processes on port $port: $pids"
			echo "$pids" | xargs kill -9 2>/dev/null || true
		fi
	done
	set -e
}

check_ollama_models() {
	local env_file="backend/.env"
	if [[ ! -f "$env_file" ]]; then
		echo "Warning: $env_file not found, skipping ollama model check"
		return 0
	fi

	if ! command -v ollama &>/dev/null; then
		echo "Error: ollama is not installed"
		exit 1
	fi

	local available
	available=$(ollama list 2>/dev/null | awk 'NR>1 {print $1}') || {
		echo "Error: failed to query ollama - is it running?"
		exit 1
	}

	local model_vars=(OLLAMA_MODEL OLLAMA_FALLBACK_MODEL SUMMARY_EVALUATOR_MODEL OLLAMA_EMBEDDING_MODEL)
	local missing=()

	for var in "${model_vars[@]}"; do
		local model
		model=$(grep -E "^${var}=" "$env_file" | cut -d= -f2- || echo "")
		[[ -z "$model" ]] && continue

		if [[ "$model" != *":"* ]]; then
			echo "Error: $var=$model is missing an explicit tag (e.g. :latest, :cloud)"
			exit 1
		fi

		if ! echo "$available" | grep -qxF "$model"; then
			missing+=("$var=$model")
		fi
	done

	if (( ${#missing[@]} > 0 )); then
		echo "Error: the following ollama models are not available:"
		for entry in "${missing[@]}"; do
			echo "  - $entry"
		done
		echo ""
		echo "Pull them with:  ollama pull <model>"
		exit 1
	fi

	echo "All ollama models verified."
}

check_ollama_models

if [[ "$mode" == "detach" ]]; then
	echo "Starting app supervisor in detached mode (log: start_app.log)"
	supervisor_pid=$(
		python3 - "$script_path" "$PWD/start_app.log" <<'PY'
import os
import subprocess
import sys

script_path = sys.argv[1]
log_path = sys.argv[2]

with open(log_path, "ab", buffering=0) as log_file:
    process = subprocess.Popen(
        ["zsh", script_path, "--detached-child"],
        cwd=os.getcwd(),
        stdin=subprocess.DEVNULL,
        stdout=log_file,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )

print(process.pid)
PY
	)
	echo "Detached supervisor PID: $supervisor_pid"
	echo "Follow startup with: tail -f start_app.log"
	exit 0
fi

echo "Cleaning up old processes on ports $frontend_port, $backend_port, and $docs_port..."
cleanup
trap cleanup EXIT INT TERM

if [[ "$mode" == "detached_child" ]]; then
	echo "Detached supervisor running for ports $frontend_port/$backend_port/$docs_port"
	echo "Starting backend on http://localhost:$backend_port (log: backend.log)"
else
	echo "Starting backend on http://localhost:$backend_port (log: backend.log, streaming enabled)"
fi
start_backend

if [[ "$mode" == "detached_child" ]]; then
	echo "Starting frontend on http://localhost:$frontend_port (log: frontend.log)"
else
	echo "Starting frontend on http://localhost:$frontend_port (log: frontend.log, streaming enabled)"
fi
start_frontend

if [[ "$mode" == "detached_child" ]]; then
	echo "Starting docs on http://localhost:$docs_port (log: docs.log)"
else
	echo "Starting docs on http://localhost:$docs_port (log: docs.log, streaming enabled)"
fi
start_docs

if ! wait_for_http "Backend" "http://localhost:$backend_port/api/health"; then
	echo "Backend failed to start. Last backend log lines:"
	tail -n 80 backend.log || true
	exit 1
fi

if ! wait_for_http "Frontend" "http://localhost:$frontend_port"; then
	echo "Frontend failed to start. Last frontend log lines:"
	tail -n 80 frontend.log || true
	exit 1
fi

if ! wait_for_http "Docs" "http://localhost:$docs_port"; then
	echo "Docs failed to start. Last docs log lines:"
	tail -n 80 docs.log || true
	exit 1
fi

echo "App is ready:"
echo "- Frontend: http://localhost:$frontend_port"
echo "- Backend:  http://localhost:$backend_port"
echo "- Docs:     http://localhost:$docs_port"

wait $backend_pid $frontend_pid $docs_pid
