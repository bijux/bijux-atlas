#!/usr/bin/env bash
set -euo pipefail
if [ "${ALLOW_NON_KIND:-0}" = "1" ] || [ "${I_KNOW_WHAT_I_AM_DOING:-0}" = "1" ]; then
  exit 0
fi
ctx="$(kubectl config current-context 2>/dev/null || true)"
if [ -z "$ctx" ]; then
  echo "kubectl context is not set" >&2
  exit 1
fi
case "$ctx" in
  kind-*|*kind*) exit 0 ;;
  *)
    echo "refusing non-kind context '$ctx' (set I_KNOW_WHAT_I_AM_DOING=1 to override)" >&2
    exit 1
    ;;
esac
