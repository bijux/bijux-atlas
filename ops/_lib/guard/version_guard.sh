#!/usr/bin/env bash
# Purpose: verify tool versions against pinned config.
set -euo pipefail

ops_version_guard() {
  local tools=()
  if [ "$#" -gt 0 ]; then
    tools=("$@")
  else
    tools=(kind k6 helm kubectl jq yq)
  fi
  python3 ./packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_tool_versions.py "${tools[@]}"
}
