#!/bin/zsh
set -euo pipefail

frontend_port=${FRONTEND_PORT:-3543}
backend_port=${BACKEND_PORT:-3544}
ports=($frontend_port $backend_port)

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

typeset -A unique_pids
for port in "${ports[@]}"; do
	for pid in ${(f)"$(collect_listener_pids "$port")"}; do
		[[ -n "$pid" ]] && unique_pids[$pid]=1
	done
done

if (( ${#unique_pids[@]} == 0 )); then
	echo "No listeners found on ports $frontend_port and $backend_port."
	exit 0
fi

pids=(${(k)unique_pids})
echo "Stopping app processes on ports $frontend_port and $backend_port: ${(j: :)pids}"
kill "${pids[@]}" >/dev/null 2>&1 || true

sleep 1
remaining_pids=()
for pid in "${pids[@]}"; do
	if kill -0 "$pid" >/dev/null 2>&1; then
		remaining_pids+=("$pid")
	fi
done

if (( ${#remaining_pids[@]} > 0 )); then
	echo "Processes still running, force stopping: ${(j: :)remaining_pids}"
	kill -9 "${remaining_pids[@]}" >/dev/null 2>&1 || true
fi

echo "Done."
