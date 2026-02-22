#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main(argv: list[str] | None = None) -> int:
    out_arg = ""
    if argv is None:
        argv = sys.argv[1:]
    if argv:
        out_arg = argv[0]
    script = f"""\
set -euo pipefail
ROOT=\"$(pwd)\"
. \"$ROOT/packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/lib/ops_common.sh\"

NS=\"${{ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_stack)}}\"
OUT=\"{out_arg if out_arg else '$ROOT/artifacts/evidence/contracts/live-snapshot.services.json'}\"
mkdir -p \"$(dirname \"$OUT\")\"

if ! kubectl cluster-info >/dev/null 2>&1; then
  echo \"live snapshot validation skipped: cluster not available\"
  exit 0
fi
if ! kubectl get ns \"$NS\" >/dev/null 2>&1; then
  echo \"live snapshot validation skipped: namespace $NS not found\"
  exit 0
fi

deploy_out=\"$(dirname \"$OUT\")/live-snapshot.deployments.json\"
triage_out=\"$(dirname \"$OUT\")/layer-drift-triage.json\"

kubectl -n \"$NS\" get svc -o json > \"$OUT\"
kubectl -n \"$NS\" get deploy -o json > \"$deploy_out\"
python3 \"$ROOT/ops/stack/tests/check_live_layer_snapshot.py\" \"$OUT\" \"$deploy_out\" \"$ROOT/ops/_meta/layer-contract.json\" \"$triage_out\"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
