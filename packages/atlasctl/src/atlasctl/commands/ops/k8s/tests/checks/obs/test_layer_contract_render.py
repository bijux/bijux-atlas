#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(r"""
rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > "$rendered"
atlas_port="$(ops_layer_port_atlas)"
required_labels_json="$(ops_layer_contract_get labels.required)"
grep -q "kind: Service" "$rendered"
grep -q "name: ${SERVICE_NAME}" "$rendered"
grep -q "port: ${atlas_port}" "$rendered"
python3 - "$rendered" "$required_labels_json" <<'PY'
import json,re,sys
rendered = open(sys.argv[1], encoding="utf-8").read()
required = json.loads(sys.argv[2])
for key in required:
    if re.search(rf"{re.escape(key)}:\s*", rendered) is None:
        raise SystemExit(f"required label missing from rendered chart: {key}")
PY
echo "layer contract render test passed"
    """, Path(__file__))

if __name__ == "__main__":
    raise SystemExit(main())
