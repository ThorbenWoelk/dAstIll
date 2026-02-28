#!/bin/zsh
set -euo pipefail

frontend_port=${FRONTEND_PORT:-3543}
backend_port=${BACKEND_PORT:-3544}
ports=($frontend_port $backend_port)

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

cleanup() {
	set +e
	for port in "${ports[@]}"; do
		ids=$(netstat -anv -p tcp \
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
			' | sort -u)
		[[ -n "$ids" ]] && echo "$ids" | xargs kill >/dev/null 2>&1 || true
	done
	set -e
}

echo "Cleaning up old processes on ports $frontend_port and $backend_port..."
cleanup
trap cleanup EXIT INT TERM

echo "Starting backend on http://localhost:$backend_port (log: backend.log, streaming enabled)"
pushd backend >/dev/null
PORT=$backend_port DATABASE_URL="../dastill.db" cargo run > >(tee ../backend.log) 2>&1 &
backend_pid=$!
popd >/dev/null

echo "Starting frontend on http://localhost:$frontend_port (log: frontend.log, streaming enabled)"
pushd frontend >/dev/null
VITE_API_BASE="http://localhost:$backend_port" bun run dev -- --host 0.0.0.0 --port $frontend_port > >(tee ../frontend.log) 2>&1 &
frontend_pid=$!
popd >/dev/null

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

echo "App is ready:"
echo "- Frontend: http://localhost:$frontend_port"
echo "- Backend:  http://localhost:$backend_port"

wait $backend_pid $frontend_pid
