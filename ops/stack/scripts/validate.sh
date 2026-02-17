#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"

find "$ROOT/stack" -name '*.yaml' -type f | while read -r f; do
  # Basic formatting drift guard: tabs are disallowed in YAML.
  if grep -n $'\t' "$f" >/dev/null; then
    echo "tab character found in YAML: $f" >&2
    exit 1
  fi
  if command -v kubectl >/dev/null 2>&1; then
    kubectl apply --dry-run=client -f "$f" >/dev/null
  fi
  if command -v kubeconform >/dev/null 2>&1; then
    kubeconform -strict -summary "$f" >/dev/null
  fi
done

echo "stack manifest validation passed"
