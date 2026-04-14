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
mobile_log_file="mobile.log"
emulator_log_file="emulator.log"
script_path=${0:A}
repo_root=${script_path:h}
link_shared_env_script="${repo_root}/scripts/link_shared_env.sh"
end_app_script="${repo_root}/end_app.sh"
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
shared_aws_dir="${shared_env_dir}/aws"
shared_aws_credentials_file="${shared_aws_dir}/credentials"
shared_aws_config_file="${shared_aws_dir}/config"
local_maintenance_mode=0

case "${LOCAL_APP_MAINTENANCE_MODE:-0}" in
	1|true|TRUE|yes|YES|on|ON)
		local_maintenance_mode=1
		;;
esac

resolve_adb_command() {
	if command -v adb >/dev/null 2>&1; then
		command -v adb
		return 0
	fi

	local candidates=(
		"${ANDROID_HOME:-}/platform-tools/adb"
		"${HOME:-}/Library/Android/sdk/platform-tools/adb"
	)

	local candidate
	for candidate in "${candidates[@]}"; do
		if [[ -n "$candidate" && -x "$candidate" ]]; then
			printf '%s' "$candidate"
			return 0
		fi
	done

	return 1
}

resolve_java_home() {
	local candidates=(
		"${JAVA_HOME:-}"
		"/Applications/Android Studio.app/Contents/jbr/Contents/Home"
	)

	local candidate
	for candidate in "${candidates[@]}"; do
		if [[ -n "$candidate" && -x "$candidate/bin/java" ]]; then
			printf '%s' "$candidate"
			return 0
		fi
	done

	return 1
}

resolve_android_home() {
	local candidates=(
		"${ANDROID_HOME:-}"
		"${HOME:-}/Library/Android/sdk"
	)

	local candidate
	for candidate in "${candidates[@]}"; do
		if [[ -n "$candidate" && -d "$candidate" ]]; then
			printf '%s' "$candidate"
			return 0
		fi
	done

	return 1
}

resolve_ndk_home() {
	if [[ -n "${NDK_HOME:-}" && -d "${NDK_HOME}" ]]; then
		printf '%s' "$NDK_HOME"
		return 0
	fi

	local android_home
	if ! android_home=$(resolve_android_home); then
		return 1
	fi

	local ndk_root="${android_home}/ndk"
	if [[ ! -d "$ndk_root" ]]; then
		return 1
	fi

	ls -1d "$ndk_root"/* 2>/dev/null | sort -V | tail -n 1
}

prepare_android_toolchain_env() {
	local resolved_java_home=""
	local resolved_android_home=""
	local resolved_ndk_home=""

	if resolved_java_home=$(resolve_java_home); then
		export JAVA_HOME="$resolved_java_home"
	fi
	if resolved_android_home=$(resolve_android_home); then
		export ANDROID_HOME="$resolved_android_home"
	fi
	if resolved_ndk_home=$(resolve_ndk_home); then
		export NDK_HOME="$resolved_ndk_home"
	fi
}

resolve_emulator_command() {
	if command -v emulator >/dev/null 2>&1; then
		command -v emulator
		return 0
	fi

	local candidates=(
		"${ANDROID_HOME:-}/emulator/emulator"
		"${HOME:-}/Library/Android/sdk/emulator/emulator"
	)

	local candidate
	for candidate in "${candidates[@]}"; do
		if [[ -n "$candidate" && -x "$candidate" ]]; then
			printf '%s' "$candidate"
			return 0
		fi
	done

	return 1
}

first_available_avd() {
	local emulator_command
	if ! emulator_command=$(resolve_emulator_command); then
		return 1
	fi

	"$emulator_command" -list-avds 2>/dev/null | awk 'NF { print; exit }'
}

connected_android_devices() {
	local adb_command
	if ! adb_command=$(resolve_adb_command); then
		return 1
	fi

	"$adb_command" devices 2>/dev/null | awk 'NR > 1 && $2 == "device" { print $1 }'
}

setup_adb_reverse() {
	local adb_command
	if ! adb_command=$(resolve_adb_command); then
		return 1
	fi

	local connected_devices
	connected_devices=$(connected_android_devices || true)
	if [[ -z "$connected_devices" ]]; then
		return 1
	fi

	local ports_to_reverse=("$frontend_port" "$backend_port" "$docs_port")
	local device
	for device in ${(f)connected_devices}; do
		[[ -z "$device" ]] && continue
		local port
		for port in "${ports_to_reverse[@]}"; do
			"$adb_command" -s "$device" reverse "tcp:${port}" "tcp:${port}" >/dev/null 2>&1 || true
		done
	done
}

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

diagnose_aws_startup_access() {
	echo "Checking AWS credentials configured for backend startup..."

	local inline_access_key
	inline_access_key=$(resolve_env_value "AWS_ACCESS_KEY_ID" "backend/.env" "$shared_backend_env_file")
	local inline_secret_key
	inline_secret_key=$(resolve_env_value "AWS_SECRET_ACCESS_KEY" "backend/.env" "$shared_backend_env_file")
	local inline_session_token
	inline_session_token=$(resolve_env_value "AWS_SESSION_TOKEN" "backend/.env" "$shared_backend_env_file")
	local explicit_credentials_file
	explicit_credentials_file=$(resolve_env_value "AWS_SHARED_CREDENTIALS_FILE" "backend/.env" "$shared_backend_env_file")
	local explicit_config_file
	explicit_config_file=$(resolve_env_value "AWS_CONFIG_FILE" "backend/.env" "$shared_backend_env_file")
	local aws_profile
	aws_profile=$(resolve_env_value "AWS_PROFILE" "backend/.env" "$shared_backend_env_file")
	if [[ -z "$aws_profile" ]]; then
		aws_profile=$(resolve_env_value "AWS_DEFAULT_PROFILE" "backend/.env" "$shared_backend_env_file")
	fi
	if [[ -z "$aws_profile" ]]; then
		aws_profile="default"
	fi

	local credentials_file=""
	if [[ -n "$explicit_credentials_file" ]]; then
		credentials_file=${~explicit_credentials_file}
	elif [[ -f "$shared_aws_credentials_file" ]]; then
		credentials_file="$shared_aws_credentials_file"
	elif [[ -n "${HOME:-}" && -f "${HOME}/.aws/credentials" ]]; then
		credentials_file="${HOME}/.aws/credentials"
	fi

	local config_file=""
	if [[ -n "$explicit_config_file" ]]; then
		config_file=${~explicit_config_file}
	elif [[ -f "$shared_aws_config_file" ]]; then
		config_file="$shared_aws_config_file"
	elif [[ -n "${HOME:-}" && -f "${HOME}/.aws/config" ]]; then
		config_file="${HOME}/.aws/config"
	fi

	if [[ -n "$inline_access_key" || -n "$inline_secret_key" || -n "$inline_session_token" ]]; then
		echo "Backend startup is using inline AWS credentials from env files."
		if [[ -n "$inline_session_token" || "${inline_access_key:u}" == ASIA* ]]; then
			echo "Hint: $shared_backend_env_file still pins temporary session credentials."
			if command -v aws >/dev/null 2>&1; then
				if [[ "$aws_profile" == "default" ]]; then
					echo "Refresh them with: aws sso login"
					echo "Then rerun: ./scripts/sync_aws_programmatic_credentials.sh"
				else
					echo "Refresh them with: aws sso login --profile $aws_profile"
					echo "Then rerun: ./scripts/sync_aws_programmatic_credentials.sh $aws_profile"
				fi
			fi
			echo "For permanent local dev, remove AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY/AWS_SESSION_TOKEN from $shared_backend_env_file"
			echo "and define a long-lived aws_access_key_id/aws_secret_access_key pair in $shared_aws_credentials_file instead."
		else
			echo "Inline credentials look like long-lived access keys, so the failure is not caused by missing AWS CLI login."
			echo "Inspect backend.log for the underlying S3/bootstrap error."
		fi
		return 0
	fi

	if [[ -n "$credentials_file" ]]; then
		echo "Backend startup will use AWS credentials file: $credentials_file (profile: $aws_profile)"
		if [[ -n "$config_file" ]]; then
			echo "AWS config file: $config_file"
		fi
		echo "If that profile still resolves temporary or expired credentials, replace it with a static aws_access_key_id/aws_secret_access_key pair for permanent local dev."
		return 0
	fi

	if command -v aws >/dev/null 2>&1; then
		local export_summary
		export_summary=$(AWS_PAGER="" aws configure export-credentials --format process 2>/dev/null | ruby -rjson -e 'j=JSON.parse(STDIN.read); ak=j["AccessKeyId"].to_s; puts "prefix=#{ak[0,4]}"; puts "session=#{j.key?("SessionToken") && !j["SessionToken"].to_s.empty? ? "yes" : "no"}"' 2>/dev/null || true)
		if [[ -n "$export_summary" ]]; then
			echo "AWS CLI currently resolves credentials for this shell, but local backend startup is not configured to use a permanent credentials file."
			echo "$export_summary"
			if [[ "$aws_profile" == "default" ]]; then
				echo "If you need a temporary fallback, run: ./scripts/sync_aws_programmatic_credentials.sh"
			else
				echo "If you need a temporary fallback, run: ./scripts/sync_aws_programmatic_credentials.sh $aws_profile"
			fi
		fi
	fi

	echo "Hint: configure permanent local AWS credentials in $shared_aws_credentials_file and keep only bucket/index settings in $shared_backend_env_file."
}

aws_startup_access_issue_detected() {
	if grep -Eiq \
		"unable to locate credentials|failed to refresh cached Login token|session has expired|token has expired|expiredtoken|refresh token has expired|AccessDeniedException" \
		backend.log 2>/dev/null; then
		return 0
	fi

	return 1
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

start_backend() {
	pushd backend >/dev/null
	PORT=$backend_port START_APP_USE_TURSO="${START_APP_USE_TURSO:-0}" cargo run --bin dastill > >(tee ../backend.log) 2>&1 &
	backend_pid=$!
	popd >/dev/null
}

start_frontend() {
	pushd frontend >/dev/null
	PUBLIC_APP_MAINTENANCE_MODE="$local_maintenance_mode" \
		VITE_API_BASE="http://localhost:$backend_port" \
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
	bun run dev > >(tee ../docs.log) 2>&1 &
	docs_pid=$!
	popd >/dev/null
}

android_device_connected() {
	connected_android_devices | grep -q .
}

wait_for_android_device() {
	local max_retries=${1:-60}
	local attempt=1

	while (( attempt <= max_retries )); do
		if android_device_connected; then
			return 0
		fi
		sleep 2
		((attempt++))
	done

	return 1
}

start_emulator_if_needed() {
	if android_device_connected; then
		return 0
	fi

	local emulator_command
	if ! emulator_command=$(resolve_emulator_command); then
		echo "Mobile shell: emulator binary not found, skipping"
		return 1
	fi

	local avd_name=${START_APP_ANDROID_AVD:-}
	if [[ -z "$avd_name" ]]; then
		avd_name=$(first_available_avd || true)
	fi

	if [[ -z "$avd_name" ]]; then
		echo "Mobile shell: no Android device/emulator detected and no AVD is available, skipping"
		return 1
	fi

	echo "Mobile shell: starting Android emulator '$avd_name' (log: $emulator_log_file)"
	"$emulator_command" -avd "$avd_name" > >(tee "$emulator_log_file") 2>&1 &
	emulator_pid=$!

	if ! wait_for_android_device 90; then
		echo "Mobile shell: emulator '$avd_name' did not become available"
		return 1
	fi

	return 0
}

resolve_tauri_android_command() {
	if cargo tauri --version >/dev/null 2>&1; then
		printf '%s' "cargo tauri android dev"
		return 0
	fi

	if command -v bunx >/dev/null 2>&1; then
		printf '%s' "bunx @tauri-apps/cli@latest android dev"
		return 0
	fi

	return 1
}

start_mobile_shell() {
	if [[ "${START_APP_SKIP_MOBILE:-}" == "1" || "${START_APP_SKIP_MOBILE:-}" == "true" || "${START_APP_SKIP_MOBILE:-}" == "TRUE" ]]; then
		echo "Mobile shell: skipped via START_APP_SKIP_MOBILE"
		return 0
	fi

	if [[ ! -d "${repo_root}/src-tauri" ]]; then
		echo "Mobile shell: src-tauri not found, skipping"
		return 0
	fi

	prepare_android_toolchain_env

	if ! start_emulator_if_needed; then
		return 0
	fi

	setup_adb_reverse || true

	local tauri_command
	if ! tauri_command=$(resolve_tauri_android_command); then
		echo "Mobile shell: neither 'cargo tauri' nor 'bunx @tauri-apps/cli' is available, skipping"
		return 0
	fi

	pushd "$repo_root" >/dev/null
	eval "$tauri_command" > >(tee "$mobile_log_file") 2>&1 &
	mobile_pid=$!
	popd >/dev/null
	echo "Starting Android shell (log: $mobile_log_file)"
}

cleanup() {
	DASTILL_SKIP_PIDS="$$ ${PPID:-}" "$end_app_script" --quiet || true
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

ensure_local_env_files
if (( local_maintenance_mode == 0 )); then
	check_ollama_models
fi

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

echo "Stopping any running dAstIll services before restart..."
cleanup
trap cleanup EXIT INT TERM

if (( local_maintenance_mode == 0 )); then
	if [[ "$mode" == "detached_child" ]]; then
		echo "Detached supervisor running for ports $frontend_port/$backend_port/$docs_port"
		echo "Starting backend on http://localhost:$backend_port (log: backend.log)"
	else
		echo "Starting backend on http://localhost:$backend_port (log: backend.log, streaming enabled)"
	fi
	start_backend

	if ! wait_for_http "Backend" "http://localhost:$backend_port/api/health" "$backend_pid"; then
		echo "Backend failed to start. Last backend log lines:"
		if aws_startup_access_issue_detected; then
			diagnose_aws_startup_access
		fi
		tail -n 80 backend.log || true
		exit 1
	fi
else
	echo "Local maintenance mode enabled; skipping backend startup."
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

if (( local_maintenance_mode == 0 )); then
	if ! require_http_status \
		"Backend workspace bootstrap" \
		"http://localhost:$backend_port/api/workspace/bootstrap?limit=20" \
		"200"; then
		echo "Backend is up, but the workspace bootstrap request failed."
		if aws_startup_access_issue_detected; then
			diagnose_aws_startup_access
			echo "Startup failed because backend AWS access is unavailable."
			echo "Last backend log lines:"
			tail -n 80 backend.log || true
			exit 1
		else
			echo "Last backend log lines:"
			tail -n 80 backend.log || true
			exit 1
		fi
	fi
fi

if ! wait_for_http "Docs" "http://localhost:$docs_port" "$docs_pid"; then
	echo "Docs failed to start. Last docs log lines:"
	tail -n 80 docs.log || true
	exit 1
fi

echo "App is ready:"
echo "- Frontend: http://localhost:$frontend_port"
echo "- Docs:     http://localhost:$docs_port"

if (( local_maintenance_mode == 0 )); then
	echo "- Backend:  http://localhost:$backend_port"
else
	echo "- Backend:  skipped (LOCAL_APP_MAINTENANCE_MODE=1)"
fi

start_mobile_shell

if [[ -n "${mobile_pid:-}" ]]; then
	if (( local_maintenance_mode == 0 )); then
		wait $backend_pid $frontend_pid $docs_pid $mobile_pid
	else
		wait $frontend_pid $docs_pid $mobile_pid
	fi
else
	if (( local_maintenance_mode == 0 )); then
		wait $backend_pid $frontend_pid $docs_pid
	else
		wait $frontend_pid $docs_pid
	fi
fi
