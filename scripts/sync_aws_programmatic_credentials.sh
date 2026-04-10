#!/bin/zsh
set -euo pipefail

script_dir=${0:A:h}
repo_root=${script_dir:h}
link_shared_env_script="${repo_root}/scripts/link_shared_env.sh"

profile="${1:-${AWS_PROFILE:-${AWS_DEFAULT_PROFILE:-}}}"

if [[ -n "${DASTILL_ENV_DIR:-}" ]]; then
	shared_env_dir="$DASTILL_ENV_DIR"
elif [[ -n "${XDG_CONFIG_HOME:-}" ]]; then
	shared_env_dir="${XDG_CONFIG_HOME}/dastill"
elif [[ -n "${HOME:-}" ]]; then
	shared_env_dir="${HOME}/.config/dastill"
else
	echo "Unable to determine shared config directory. Set DASTILL_ENV_DIR or HOME."
	exit 1
fi

shared_backend_env_file="${shared_env_dir}/backend.env"

if ! command -v aws >/dev/null 2>&1; then
	echo "AWS CLI is required to export local programmatic credentials."
	exit 1
fi

"$link_shared_env_script" >/dev/null

export_command=(aws configure export-credentials --format env-no-export)
if [[ -n "$profile" ]]; then
	export_command+=(--profile "$profile")
fi

exported="$("${export_command[@]}")"

aws_access_key_id=""
aws_secret_access_key=""
aws_session_token=""

while IFS='=' read -r key value; do
	case "$key" in
	AWS_ACCESS_KEY_ID)
		aws_access_key_id="$value"
		;;
	AWS_SECRET_ACCESS_KEY)
		aws_secret_access_key="$value"
		;;
	AWS_SESSION_TOKEN)
		aws_session_token="$value"
		;;
	esac
done <<< "$exported"

if [[ -z "$aws_access_key_id" || -z "$aws_secret_access_key" ]]; then
	echo "Failed to export AWS programmatic credentials."
	exit 1
fi

update_env_key() {
	local file=$1
	local key=$2
	local value=$3
	local tmp_file
	tmp_file=$(mktemp "${file}.XXXXXX")

	awk -v key="$key" -v value="$value" '
	BEGIN { updated = 0 }
	index($0, key "=") == 1 {
		if (length(value) > 0) {
			print key "=" value
		}
		updated = 1
		next
	}
	{ print }
	END {
		if (!updated && length(value) > 0) {
			print key "=" value
		}
	}
	' "$file" > "$tmp_file"

	mv "$tmp_file" "$file"
}

update_env_key "$shared_backend_env_file" "AWS_ACCESS_KEY_ID" "$aws_access_key_id"
update_env_key "$shared_backend_env_file" "AWS_SECRET_ACCESS_KEY" "$aws_secret_access_key"
update_env_key "$shared_backend_env_file" "AWS_SESSION_TOKEN" "$aws_session_token"

if [[ -n "$profile" ]]; then
	echo "Synced AWS programmatic credentials for profile '$profile' into $shared_backend_env_file"
else
	echo "Synced AWS programmatic credentials into $shared_backend_env_file"
fi
