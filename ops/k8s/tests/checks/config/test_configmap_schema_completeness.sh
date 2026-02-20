#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
cd "$ROOT"

tmpl_keys="$(mktemp)"
doc_keys="$(mktemp)"
val_keys="$(mktemp)"
trap 'rm -f "$tmpl_keys" "$doc_keys" "$val_keys"' EXIT

awk '/^[[:space:]]+ATLAS_[A-Z0-9_]+:/{gsub(":","",$1); print $1}' ops/k8s/charts/bijux-atlas/templates/configmap.yaml | sort -u >"$tmpl_keys"
grep -oE '`ATLAS_[A-Z0-9_]+`' docs/operations/config.md | tr -d '`' | sort -u >"$doc_keys"
grep -oE '`values\.[a-zA-Z0-9_.-]+`' docs/operations/k8s/values.md | tr -d '`' | sort -u >"$val_keys"

missing_docs="$(comm -23 "$tmpl_keys" "$doc_keys" || true)"
if [ -n "$missing_docs" ]; then
  echo "configmap schema completeness failed: missing config key docs in docs/operations/config.md" >&2
  echo "$missing_docs" >&2
  exit 1
fi

while read -r top; do
  [ -z "$top" ] && continue
  if ! grep -qx "values.${top}" "$val_keys"; then
    echo "configmap schema completeness failed: missing values.${top} in docs/operations/k8s/values.md" >&2
    exit 1
  fi
done < <(grep -oE '\.Values\.[a-zA-Z0-9_]+' ops/k8s/charts/bijux-atlas/templates/configmap.yaml | sed 's/\.Values\.//' | cut -d'.' -f1 | sort -u)

echo "configmap schema completeness passed"
