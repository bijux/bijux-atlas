#!/usr/bin/env bash
set -euo pipefail

# Shared script error helpers (structured stderr lines).
err() {
  local code="$1"
  shift
  printf '{"error_code":"%s","message":"%s"}\n' "$code" "$*" >&2
}
