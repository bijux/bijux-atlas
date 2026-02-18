#!/usr/bin/env bash
# Purpose: guard kubectl context by expected profile.
set -euo pipefail

ops_context_guard() {
  local profile="${1:-kind}"
  local ctx
  ctx="$(kubectl config current-context 2>/dev/null || true)"
  case "$profile" in
    kind)
      case "$ctx" in
        kind-*|*kind*) return 0 ;;
      esac
      echo "invalid kubectl context '$ctx' for profile=kind" >&2
      return 2
      ;;
    perf)
      if [ -z "$ctx" ]; then
        echo "missing kubectl context for profile=perf" >&2
        return 2
      fi
      return 0
      ;;
    *)
      echo "unknown profile '$profile'" >&2
      return 2
      ;;
  esac
}
