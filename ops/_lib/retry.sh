#!/usr/bin/env bash
# Purpose: retry wrapper with bounded attempts and backoff.
# Inputs: attempts, sleep-seconds, command args.
# Outputs: command exit status after retries.
set -euo pipefail

ops_retry() {
  local attempts="$1"
  local sleep_seconds="$2"
  shift 2
  local n=1
  while true; do
    if "$@"; then
      return 0
    fi
    if [ "$n" -ge "$attempts" ]; then
      echo "command failed after $attempts attempts: $*" >&2
      return 1
    fi
    n=$((n + 1))
    sleep "$sleep_seconds"
  done
}
