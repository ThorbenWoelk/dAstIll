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

mkdir -p "$shared_env_dir"
mkdir -p "$shared_env_dir/aws"

link_env_file() {
	local worktree_relative_path=$1
	local shared_filename=$2
	local template_path=${3:-}
	local worktree_path="${repo_root}/${worktree_relative_path}"
	local shared_path="${shared_env_dir}/${shared_filename}"

	mkdir -p "${worktree_path:h}"

	if [[ -L "$worktree_path" ]]; then
		local current_target
		current_target=$(readlink "$worktree_path")
		if [[ "$current_target" == "$shared_path" ]]; then
			echo "${worktree_relative_path} already points to ${shared_path}"
			return 0
		fi
		rm "$worktree_path"
	fi

	if [[ -f "$worktree_path" && ! -e "$shared_path" ]]; then
		mv "$worktree_path" "$shared_path"
		echo "Moved ${worktree_relative_path} to ${shared_path}"
	elif [[ -f "$worktree_path" && -e "$shared_path" ]]; then
		if cmp -s "$worktree_path" "$shared_path"; then
			rm "$worktree_path"
		else
			echo "Refusing to overwrite ${shared_path} with a different ${worktree_relative_path}"
			exit 1
		fi
	elif [[ ! -e "$shared_path" && -n "$template_path" && -f "${repo_root}/${template_path}" ]]; then
		cp "${repo_root}/${template_path}" "$shared_path"
		echo "Created ${shared_path} from ${template_path}"
	fi

	ln -s "$shared_path" "$worktree_path"
	echo "Linked ${worktree_relative_path} -> ${shared_path}"
}

link_env_file "backend/.env" "backend.env" "backend/.env.example"
link_env_file "frontend/.env" "frontend.env" "frontend/.env.example"
