#!/usr/bin/env bash
# Purpose: timeout wrapper to avoid unbounded waits.
# Inputs: timeout-seconds, command args.
# Outputs: command exit status (timed out if supported).
set -euo pipefail

ops_timeout_run() {
  local timeout_seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$timeout_seconds" "$@"
    return $?
  fi
  "$@"
}
