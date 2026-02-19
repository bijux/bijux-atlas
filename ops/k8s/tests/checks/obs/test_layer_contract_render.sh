#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
# shellcheck source=/dev/null
. "$ROOT/ops/_lib/k8s-test-common.sh"

rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > "$rendered"

atlas_port="$(ops_layer_port_atlas)"
required_labels_json="$(ops_layer_contract_get labels.required)"

grep -q "kind: Service" "$rendered"
grep -q "name: ${SERVICE_NAME}" "$rendered"
grep -q "port: ${atlas_port}" "$rendered"

python3 - "$rendered" "$required_labels_json" <<'PY'
import json
import re
import sys

rendered = open(sys.argv[1], encoding="utf-8").read()
required = json.loads(sys.argv[2])
for key in required:
    if re.search(rf"{re.escape(key)}:\s*", rendered) is None:
        raise SystemExit(f"required label missing from rendered chart: {key}")
PY

echo "layer contract render test passed"
