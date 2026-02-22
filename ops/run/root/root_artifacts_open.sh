#!/usr/bin/env bash
set -euo pipefail

run_id="${RUN_ID:-$(cat artifacts/evidence/latest-run-id.txt 2>/dev/null || true)}"
if [ -z "${run_id}" ]; then
  echo "no recent run found in artifacts/evidence/latest-run-id.txt"
  exit 1
fi
path="artifacts/evidence/ci/${run_id}"
if [ ! -d "${path}" ]; then
  echo "evidence path not found: ${path}"
  exit 1
fi
echo "${path}"
