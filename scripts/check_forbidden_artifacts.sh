#!/bin/bash

set -euo pipefail

forbidden=()

while IFS= read -r -d '' path; do
  case "$path" in
    backend/backend-sa-key.json|backend/backend-wif-token.jwt|terraform/tfplan|terraform/tfplan-*|*.tfstate|*.tfstate.*|*.tfvars|*.tfvars.json|*.auto.tfvars|*.auto.tfvars.json)
      forbidden+=("$path")
      ;;
  esac
done < <(git ls-files -z)

if [[ ${#forbidden[@]} -gt 0 ]]; then
  echo "Forbidden tracked artifacts detected:"
  printf ' - %s\n' "${forbidden[@]}"
  echo "Remove them from git tracking before merging."
  exit 1
fi
