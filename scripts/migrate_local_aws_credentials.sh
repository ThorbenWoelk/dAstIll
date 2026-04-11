#!/bin/zsh
set -euo pipefail

script_dir=${0:A:h}
repo_root=${script_dir:h}

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
shared_aws_dir="${shared_env_dir}/aws"
shared_aws_credentials_file="${shared_aws_dir}/credentials"
shared_aws_config_file="${shared_aws_dir}/config"

mkdir -p "$shared_aws_dir"

read_env_value() {
	local key=$1
	if [[ ! -f "$shared_backend_env_file" ]]; then
		return 0
	fi
	local line
	line=$(grep -E "^${key}=" "$shared_backend_env_file" | head -n1 || true)
	line=${line#*=}
	line=${line#\"}
	line=${line%\"}
	printf '%s' "$line"
}

write_static_credentials_file() {
	local access_key=$1
	local secret_key=$2
	local region=$3

	cat >"$shared_aws_credentials_file" <<EOF
[default]
aws_access_key_id = $access_key
aws_secret_access_key = $secret_key
EOF

	cat >"$shared_aws_config_file" <<EOF
[default]
region = $region
EOF
}

strip_inline_aws_creds_from_backend_env() {
	if [[ ! -f "$shared_backend_env_file" ]]; then
		return 0
	fi
	local tmp_file
	tmp_file=$(mktemp)
	grep -Ev '^(AWS_ACCESS_KEY_ID|AWS_SECRET_ACCESS_KEY|AWS_SESSION_TOKEN)=' "$shared_backend_env_file" >"$tmp_file" || true
	mv "$tmp_file" "$shared_backend_env_file"
}

access_key=$(read_env_value "AWS_ACCESS_KEY_ID")
secret_key=$(read_env_value "AWS_SECRET_ACCESS_KEY")
session_token=$(read_env_value "AWS_SESSION_TOKEN")
aws_region=$(read_env_value "AWS_REGION")
if [[ -z "$aws_region" ]]; then
	aws_region="eu-central-1"
fi

if [[ -z "$access_key" || -z "$secret_key" ]]; then
	echo "No inline AWS access key pair found in $shared_backend_env_file."
	echo "Nothing to migrate."
	exit 0
fi

if [[ -n "$session_token" || "${access_key:u}" == ASIA* ]]; then
	echo "Refusing to migrate temporary AWS session credentials."
	echo "Current backend env uses an STS-style access key (${access_key[1,4]}...) and/or AWS_SESSION_TOKEN."
	echo "For permanent local dev, create a long-lived aws_access_key_id/aws_secret_access_key pair in:"
	echo "  $shared_aws_credentials_file"
	echo "Then remove AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY/AWS_SESSION_TOKEN from:"
	echo "  $shared_backend_env_file"
	exit 1
fi

write_static_credentials_file "$access_key" "$secret_key" "$aws_region"
strip_inline_aws_creds_from_backend_env

echo "Migrated inline AWS credentials from $shared_backend_env_file"
echo "to the shared credentials files:"
echo "  $shared_aws_credentials_file"
echo "  $shared_aws_config_file"
echo "Removed inline AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY/AWS_SESSION_TOKEN from backend.env."
