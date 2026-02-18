#!/usr/bin/env bash
set -euo pipefail
if [ "${ALLOW_NON_KIND:-0}" = "1" ]; then
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
    echo "refusing non-kind context '$ctx' (set ALLOW_NON_KIND=1 to override)" >&2
    exit 1
    ;;
esac
