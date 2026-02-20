#!/usr/bin/env bash
# Purpose: canonical ops exit codes from SSOT error registry.
set -euo pipefail

OPS_ERR_CONFIG=10
OPS_ERR_CONTEXT=11
OPS_ERR_VERSION=12
OPS_ERR_PREREQ=13
OPS_ERR_TIMEOUT=14
OPS_ERR_VALIDATION=15
OPS_ERR_ARTIFACT=16
OPS_ERR_DOCS=17
OPS_ERR_INTERNAL=99

ops_fail() {
  local code="$1"
  shift
  printf '%s\n' "${*:-ops failure}" >&2
  exit "$code"
}
