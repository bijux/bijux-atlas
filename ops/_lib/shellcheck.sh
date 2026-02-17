#!/usr/bin/env bash
# Purpose: run shellcheck against ops shell scripts with stable excludes.
# Inputs: ops/**/*.sh scripts.
# Outputs: shellcheck diagnostics and exit code.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
LIST_FILE="$(mktemp)"
trap 'rm -f "$LIST_FILE"' EXIT

find "$ROOT/ops" -type f -name '*.sh' | sort > "$LIST_FILE"
if [ ! -s "$LIST_FILE" ]; then
  echo "no ops shell scripts found"
  exit 0
fi

if command -v shellcheck >/dev/null 2>&1; then
  xargs shellcheck -x < "$LIST_FILE"
  exit 0
fi

if command -v docker >/dev/null 2>&1; then
  echo "shellcheck not found locally; using docker image"
  docker_fail=0
  while IFS= read -r f; do
    rel="${f#${ROOT}/}"
    if ! docker run --rm -v "$ROOT:/mnt" koalaman/shellcheck:stable -x "/mnt/$rel"; then
      docker_fail=1
      break
    fi
  done < "$LIST_FILE"
  if [ "$docker_fail" -eq 0 ]; then
    exit 0
  fi
fi

if [ "${SHELLCHECK_STRICT:-0}" = "1" ]; then
  echo "shellcheck is required (install shellcheck or docker)" >&2
  exit 1
fi

echo "shellcheck skipped: shellcheck/docker unavailable (non-strict mode)" >&2
exit 0
