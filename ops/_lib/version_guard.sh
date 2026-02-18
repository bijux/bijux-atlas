#!/usr/bin/env bash
# Purpose: verify tool versions against pinned config.
set -euo pipefail

ops_version_guard() {
  local tools=("${@:-kind k6 helm kubectl jq yq}")
  python3 ./scripts/layout/check_tool_versions.py "${tools[@]}"
}
