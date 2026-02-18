#!/usr/bin/env bash
set -euo pipefail
ns="${ATLAS_NS:-${ATLAS_E2E_NAMESPACE:-}}"
if [ -z "$ns" ]; then
  echo "ATLAS_NS/ATLAS_E2E_NAMESPACE is empty" >&2
  exit 1
fi
case "$ns" in
  atlas-ops-*) exit 0 ;;
  *)
    echo "namespace must match atlas-ops-* (got: $ns)" >&2
    exit 1
    ;;
esac
