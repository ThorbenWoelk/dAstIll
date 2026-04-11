#!/bin/zsh
set -euo pipefail

quiet=0
case "${1:-}" in
	"")
		;;
	--quiet)
		quiet=1
		;;
	*)
		echo "Usage: ./end_app.sh [--quiet]"
		exit 1
		;;
esac

frontend_port=${FRONTEND_PORT:-3543}
backend_port=${BACKEND_PORT:-3544}
docs_port=${DOCS_PORT:-4173}
ports=($frontend_port $backend_port $docs_port)
script_path=${0:A}
repo_root=${script_path:h}
android_app_id="com.dastill.app"
skip_pids=(${=DASTILL_SKIP_PIDS:-})

log() {
	if (( quiet == 0 )); then
		echo "$@"
	fi
}

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

process_is_running() {
	local pid=$1
	set +e
	kill -0 "$pid" 2>/dev/null
	local exit_code=$?
	set -e
	return $exit_code
}

collect_listener_pids() {
	local port=$1

	if command -v lsof >/dev/null 2>&1; then
		lsof -nP -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null | sort -u
		return
	fi

	netstat -anv -p tcp \
		| awk -v target_port=".$port" '
			$6 == "LISTEN" && $4 ~ (target_port "$") {
				for (i = 1; i <= NF; i++) {
					if ($i ~ /:[0-9]+$/) {
						split($i, parts, ":");
						print parts[2];
						break;
					}
				}
			}
		' | sort -u
}

collect_matching_pids() {
	local pattern=$1
	ps -Ao pid=,command= \
		| awk -v pat="$pattern" '
			index($0, pat) {
				pid = $1
				if (pid ~ /^[0-9]+$/) print pid
			}
		' | sort -u
}

typeset -A unique_pids
typeset -A skip_pid_map

for pid in "${skip_pids[@]}"; do
	[[ "$pid" =~ ^[0-9]+$ ]] || continue
	skip_pid_map[$pid]=1
done

for port in "${ports[@]}"; do
	for pid in ${(f)"$(collect_listener_pids "$port")"}; do
		[[ -n "$pid" ]] && unique_pids[$pid]=1
	done
done

match_patterns=(
	"$repo_root/start_app.sh"
	"/repos/dAstIll/start_app.sh"
	"cargo tauri android dev"
	"cargo-tauri tauri android dev"
	"@tauri-apps/cli@latest android dev"
	"$repo_root/src-tauri/gen/android/gradlew"
	"tauri android android-studio-script"
	"target/debug/dastill"
	"vite dev --host 0.0.0.0 --port ${frontend_port}"
	"vitepress dev . --host 0.0.0.0 --port ${docs_port}"
)

for pattern in "${match_patterns[@]}"; do
	for pid in ${(f)"$(collect_matching_pids "$pattern")"}; do
		[[ -n "$pid" ]] && unique_pids[$pid]=1
	done
done

pids=(${(k)unique_pids})

filtered_pids=()
for pid in "${pids[@]}"; do
	if [[ -n "${skip_pid_map[$pid]:-}" ]]; then
		continue
	fi
	filtered_pids+=("$pid")
done
pids=("${filtered_pids[@]}")

if (( ${#pids[@]} > 0 )); then
	log "Stopping dAstIll processes: ${(j: :)pids}"
	kill "${pids[@]}" >/dev/null 2>&1 || true

	sleep 1
	remaining_pids=()
	for pid in "${pids[@]}"; do
		if process_is_running "$pid"; then
			remaining_pids+=("$pid")
		fi
	done

	if (( ${#remaining_pids[@]} > 0 )); then
		log "Force stopping remaining processes: ${(j: :)remaining_pids}"
		kill -9 "${remaining_pids[@]}" >/dev/null 2>&1 || true
	fi
else
	log "No local dAstIll processes found."
fi

adb_command=""
if adb_command=$(resolve_adb_command); then
	connected_devices=$("$adb_command" devices 2>/dev/null | awk 'NR > 1 && $2 == "device" { print $1 }')
	if [[ -n "$connected_devices" ]]; then
		log "Stopping Android app ${android_app_id} on connected device(s)"
		while IFS= read -r device; do
			[[ -z "$device" ]] && continue
			"$adb_command" -s "$device" shell am force-stop "$android_app_id" >/dev/null 2>&1 || true
			if [[ "$device" == emulator-* ]]; then
				log "Stopping Android emulator ${device}"
				"$adb_command" -s "$device" emu kill >/dev/null 2>&1 || true
			fi
		done <<< "$connected_devices"
	fi
fi

log "Done."
