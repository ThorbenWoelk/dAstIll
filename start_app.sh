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
repo_root=${script_path:h}
link_shared_env_script="${repo_root}/scripts/link_shared_env.sh"
if [[ -n "${DASTILL_ENV_DIR:-}" ]]; then
	shared_env_dir="$DASTILL_ENV_DIR"
elif [[ -n "${XDG_CONFIG_HOME:-}" ]]; then
	shared_env_dir="${XDG_CONFIG_HOME}/dastill"
elif [[ -n "${HOME:-}" ]]; then
	shared_env_dir="${HOME}/.config/dastill"
else
	shared_env_dir="$PWD/.config/dastill"
fi
shared_backend_env_file="${shared_env_dir}/backend.env"
shared_frontend_env_file="${shared_env_dir}/frontend.env"
typeset -gA initial_env_keys

process_is_running() {
	local pid=$1
	local exit_code=0

	set +e
	kill -0 "$pid" 2>/dev/null
	exit_code=$?
	set -e

	return $exit_code
}

wait_for_http() {
	local name=$1
	local url=$2
	local pid=${3:-}
	local max_retries=${4:-30}
	local attempt=1

	while (( attempt <= max_retries )); do
		if curl -fsS "$url" >/dev/null 2>&1; then
			return 0
		fi
		if [[ -n "$pid" ]] && ! process_is_running "$pid"; then
			echo "$name exited before becoming ready at $url"
			return 1
		fi
		sleep 1
		((attempt++))
	done

	echo "$name did not become ready at $url"
	return 1
}

require_http_status() {
	local name=$1
	local url=$2
	local expected_status=$3
	local http_status

	set +e
	http_status=$(curl -sS -o /dev/null -w "%{http_code}" "$url")
	local curl_exit=$?
	set -e

	if [[ $curl_exit -ne 0 ]]; then
		echo "$name could not be reached at $url"
		return 1
	fi

	if [[ "$http_status" != "$expected_status" ]]; then
		echo "$name returned HTTP $http_status at $url (expected $expected_status)"
		return 1
	fi

	return 0
}

capture_initial_env_keys() {
	initial_env_keys=()
	while IFS='=' read -r key _; do
		initial_env_keys[$key]=1
	done < <(env)
}

trim_whitespace() {
	local value=$1
	value="${value#"${value%%[![:space:]]*}"}"
	value="${value%"${value##*[![:space:]]}"}"
	printf '%s' "$value"
}

read_env_file_value() {
	local env_file=$1
	local key=$2
	local value=""

	if [[ ! -f "$env_file" ]]; then
		return 0
	fi

	value=$(grep -E "^${key}=" "$env_file" | head -n1 | cut -d= -f2- || true)
	printf '%s' "$value"
	return 0
}

resolve_env_value() {
	local key=$1
	local local_env_file=${2:-}
	local shared_env_file=${3:-}

	if (( ${+parameters[$key]} )); then
		printf '%s' "${(P)key}"
		return 0
	fi

	local value=""
	if [[ -n "$local_env_file" ]]; then
		value=$(read_env_file_value "$local_env_file" "$key")
	fi
	if [[ -n "$value" ]]; then
		printf '%s' "$value"
		return 0
	fi
	if [[ -n "$shared_env_file" ]]; then
		value=$(read_env_file_value "$shared_env_file" "$key")
	fi
	printf '%s' "$value"
	return 0
}

export_env_file_preserving_shell() {
	local env_file=$1
	local line=""

	if [[ ! -f "$env_file" ]]; then
		return 0
	fi

	while IFS= read -r line || [[ -n "$line" ]]; do
		[[ -z "$line" ]] && continue
		[[ "$line" == \#* ]] && continue
		[[ "$line" != *=* ]] && continue

		local key=${line%%=*}
		local value=${line#*=}
		key=$(trim_whitespace "$key")
		value=$(trim_whitespace "$value")

		if [[ -z "$key" || "$key" == \#* || -n ${initial_env_keys[$key]-} ]]; then
			continue
		fi
		if [[ "$value" == \"*\" && "$value" == *\" ]]; then
			value=${value#\"}
			value=${value%\"}
		fi

		export "$key=$value"
	done < "$env_file"
}

ensure_local_env_files() {
	local missing_env=0

	[[ -f "$shared_backend_env_file" ]] || missing_env=1
	[[ -f "$shared_frontend_env_file" ]] || missing_env=1
	[[ -f "backend/.env" ]] || missing_env=1
	[[ -f "frontend/.env" ]] || missing_env=1

	if (( missing_env == 0 )); then
		return 0
	fi

	echo "Local env files missing; running scripts/link_shared_env.sh"
	"$link_shared_env_script"

	if [[ ! -f "$shared_backend_env_file" || ! -f "$shared_frontend_env_file" || ! -f "backend/.env" || ! -f "frontend/.env" ]]; then
		echo "Error: failed to set up local env files under backend/.env, frontend/.env, or ${shared_env_dir}"
		exit 1
	fi
}

prepare_frontend_env() {
	export_env_file_preserving_shell "$shared_frontend_env_file"
	export_env_file_preserving_shell "frontend/.env"
}

start_backend() {
	pushd backend >/dev/null
	local summary_model
	summary_model=$(resolve_env_value "OLLAMA_SUMMARY_MODEL" ".env" "$shared_backend_env_file")
	if [[ -z "$summary_model" ]]; then
		summary_model=$(resolve_env_value "OLLAMA_MODEL" ".env" "$shared_backend_env_file")
	fi

	local default_chat_model
	default_chat_model=$(resolve_env_value "OLLAMA_DEFAULT_CHAT_MODEL" ".env" "$shared_backend_env_file")
	if [[ -z "$default_chat_model" ]]; then
		default_chat_model=$(resolve_env_value "OLLAMA_CHAT_MODEL" ".env" "$shared_backend_env_file")
	fi

	if [[ -n "$summary_model" ]]; then
		export OLLAMA_SUMMARY_MODEL="$summary_model"
	fi
	if [[ -n "$default_chat_model" ]]; then
		export OLLAMA_DEFAULT_CHAT_MODEL="$default_chat_model"
	fi

	local use_turso="${START_APP_USE_TURSO:-}"
	if [[ "$use_turso" == "1" || "$use_turso" == "true" || "$use_turso" == "TRUE" ]]; then
		echo "Backend search index: using configured Turso/libSQL replica"
	else
		export TURSO_DB_URL=""
		export TURSO_AUTH_TOKEN=""
		echo "Backend search index: using local libSQL fallback (set START_APP_USE_TURSO=1 to use Turso)"
	fi

	PORT=$backend_port cargo run --bin dastill > >(tee ../backend.log) 2>&1 &
	backend_pid=$!
	popd >/dev/null
}

start_frontend() {
	prepare_frontend_env
	pushd frontend >/dev/null
	local backend_proxy_token
	backend_proxy_token=$(resolve_env_value "BACKEND_PROXY_TOKEN" "../backend/.env" "$shared_backend_env_file")
	if [[ -z "$backend_proxy_token" ]]; then
		backend_proxy_token="local-dev-backend-proxy-token"
	fi
	BACKEND_API_BASE="http://localhost:$backend_port" \
		BACKEND_PROXY_TOKEN="$backend_proxy_token" \
		bun --no-env-file run dev -- --host 0.0.0.0 --port $frontend_port > >(tee ../frontend.log) 2>&1 &
	frontend_pid=$!
	popd >/dev/null
}

start_docs() {
	pushd docs >/dev/null
	if [[ ! -x "./node_modules/.bin/vitepress" ]]; then
		echo "Docs dependencies missing; running bun install --frozen-lockfile"
		bun install --frozen-lockfile
	fi
	bunx vitepress dev . --host 0.0.0.0 --port $docs_port > >(tee ../docs.log) 2>&1 &
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
	if ! command -v ollama &>/dev/null; then
		local model_vars=(OLLAMA_SUMMARY_MODEL OLLAMA_DEFAULT_CHAT_MODEL OLLAMA_FALLBACK_MODEL SUMMARY_EVALUATOR_MODEL OLLAMA_EMBEDDING_MODEL)
		for var in "${model_vars[@]}"; do
			local configured_model
			configured_model=$(resolve_env_value "$var" "backend/.env" "$shared_backend_env_file")
			if [[ -n "$configured_model" ]]; then
				echo "Error: ollama is not installed"
				exit 1
			fi
		done
		if [[ -f "$shared_backend_env_file" || -f "backend/.env" ]]; then
			echo "Ollama: not installed, but no local model variables are configured"
		fi
		return 0
	fi

	local available
	available=$(ollama list 2>/dev/null | awk 'NR>1 {print $1}') || {
		echo "Error: failed to query ollama - is it running?"
		exit 1
	}

	local model_vars=(OLLAMA_SUMMARY_MODEL OLLAMA_DEFAULT_CHAT_MODEL OLLAMA_FALLBACK_MODEL SUMMARY_EVALUATOR_MODEL OLLAMA_EMBEDDING_MODEL)
	local missing=()
	local verified=0
	local checked_files=()
	if [[ -f "$shared_backend_env_file" ]]; then
		checked_files+=("$shared_backend_env_file")
	fi
	if [[ -f "backend/.env" ]]; then
		checked_files+=("backend/.env")
	fi
	if (( ${#checked_files[@]} == 0 )); then
		echo "Ollama: no backend env file found under backend/.env or $shared_backend_env_file"
	else
		echo "Ollama: checking models from ${checked_files[*]}"
	fi

	for var in "${model_vars[@]}"; do
		local model
		model=$(resolve_env_value "$var" "backend/.env" "$shared_backend_env_file")
		[[ -z "$model" ]] && continue

		if [[ "$model" != *":"* ]]; then
			echo "Error: $var=$model is missing an explicit tag (e.g. :latest, :cloud)"
			exit 1
		fi

		if ! echo "$available" | grep -qxF "$model"; then
			missing+=("$var=$model")
		else
			printf '  %-28s %s\n' "$var" "$model"
			verified=$((verified + 1))
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

	if (( verified == 0 )); then
		echo "  (no OLLAMA_* model variables set; nothing to verify)"
	fi
	echo "Ollama: ok ($verified model(s) present locally)"
}

capture_initial_env_keys
ensure_local_env_files
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

if ! wait_for_http "Backend" "http://localhost:$backend_port/api/health" "$backend_pid"; then
	echo "Backend failed to start. Last backend log lines:"
	tail -n 80 backend.log || true
	exit 1
fi

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

if ! wait_for_http "Frontend" "http://localhost:$frontend_port" "$frontend_pid"; then
	echo "Frontend failed to start. Last frontend log lines:"
	tail -n 80 frontend.log || true
	exit 1
fi

if ! require_http_status \
	"Frontend workspace bootstrap" \
	"http://localhost:$frontend_port/api/workspace/bootstrap?limit=20" \
	"200"; then
	echo "Frontend is up, but the proxied workspace bootstrap request failed. Last frontend log lines:"
	tail -n 80 frontend.log || true
	echo "Last backend log lines:"
	tail -n 80 backend.log || true
	exit 1
fi

if ! wait_for_http "Docs" "http://localhost:$docs_port" "$docs_pid"; then
	echo "Docs failed to start. Last docs log lines:"
	tail -n 80 docs.log || true
	exit 1
fi

echo "App is ready:"
echo "- Frontend: http://localhost:$frontend_port"
echo "- Backend:  http://localhost:$backend_port"
echo "- Docs:     http://localhost:$docs_port"

wait $backend_pid $frontend_pid $docs_pid
