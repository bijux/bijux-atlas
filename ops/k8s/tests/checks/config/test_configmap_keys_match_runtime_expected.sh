#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
cd "$ROOT"

tmpl_keys="$(mktemp)"
runtime_keys="$(mktemp)"
trap 'rm -f "$tmpl_keys" "$runtime_keys"' EXIT

awk '/^[[:space:]]+ATLAS_[A-Z0-9_]+:/{gsub(":","",$1); print $1}' ops/k8s/charts/bijux-atlas/templates/configmap.yaml | sort -u >"$tmpl_keys"
python3 - <<'PY' >"$runtime_keys"
import json
from pathlib import Path
doc = json.loads((Path("docs/contracts/CONFIG_KEYS.json")).read_text(encoding="utf-8"))
for key in sorted(doc.get("keys", [])):
    print(key)
PY

unknown="$(comm -23 "$tmpl_keys" "$runtime_keys" || true)"
if [ -n "$unknown" ]; then
  echo "configmap keys/runtime contract failed: keys not declared in docs/contracts/CONFIG_KEYS.json" >&2
  echo "$unknown" >&2
  exit 1
fi

echo "configmap keys match runtime expected contract passed"
