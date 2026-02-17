#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"

KUBECTL_LOCAL_VALIDATE=0
if command -v kubectl >/dev/null 2>&1 && kubectl version --client >/dev/null 2>&1; then
  if kubectl cluster-info >/dev/null 2>&1; then
    KUBECTL_LOCAL_VALIDATE=1
  fi
fi

find "$ROOT/stack" -name '*.yaml' -type f | while read -r f; do
  # Basic formatting drift guard: tabs are disallowed in YAML.
  if grep -n $'\t' "$f" >/dev/null; then
    echo "tab character found in YAML: $f" >&2
    exit 1
  fi
  if [ "$KUBECTL_LOCAL_VALIDATE" -eq 1 ]; then
    # Keep validation local/offline: do not require a live cluster API.
    kubectl apply --dry-run=client --validate=false -f "$f" >/dev/null
  fi
  if command -v kubeconform >/dev/null 2>&1; then
    kubeconform -strict -summary "$f" >/dev/null
  fi
done

echo "stack manifest validation passed"
