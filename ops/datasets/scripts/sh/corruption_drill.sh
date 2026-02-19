#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: simulate dataset corruption and mark quarantine on detection.
# stability: public
# called-by: make ops-drill-corruption-dataset
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../../.." && pwd)"
STORE_ROOT="${ATLAS_E2E_STORE_ROOT:-$ROOT/artifacts/e2e-store}"
QDIR="$ROOT/artifacts/e2e-datasets/quarantine"
mkdir -p "$QDIR"
target="$(find "$STORE_ROOT" -type f -name 'manifest.json' | head -n1 || true)"
[ -n "$target" ] || { echo "no dataset manifest found under $STORE_ROOT" >&2; exit 1; }
q="$QDIR/$(basename "$(dirname "$target")").bad"
report="$QDIR/corruption-drill-report.json"
if [ -f "$q" ]; then
  echo "quarantine marker present, skipping repeated retry: $q"
  python3 - "$target" "$q" "$report" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
target, marker, report = sys.argv[1:4]
payload = {
  "schema_version": 1,
  "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
  "status": "skipped",
  "target_manifest": target,
  "quarantine_marker": marker,
}
schema = json.loads(Path("ops/_schemas/datasets/corruption-drill-report.schema.json").read_text(encoding="utf-8"))
for key in schema.get("required", []):
  if key not in payload:
    raise SystemExit(f"corruption report missing required key: {key}")
open(report, "w", encoding="utf-8").write(json.dumps(payload, indent=2) + "\n")
PY
  exit 0
fi
printf '\n#corrupted\n' >> "$target"
if ! jq . "$target" >/dev/null 2>&1; then
  echo "$(date -u +%FT%TZ) $target" > "$q"
  python3 - "$target" "$q" "$report" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
target, marker, report = sys.argv[1:4]
payload = {
  "schema_version": 1,
  "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
  "status": "quarantined",
  "target_manifest": target,
  "quarantine_marker": marker,
}
schema = json.loads(Path("ops/_schemas/datasets/corruption-drill-report.schema.json").read_text(encoding="utf-8"))
for key in schema.get("required", []):
  if key not in payload:
    raise SystemExit(f"corruption report missing required key: {key}")
open(report, "w", encoding="utf-8").write(json.dumps(payload, indent=2) + "\n")
PY
  echo "corruption detected and quarantined: $q"
  exit 0
fi
python3 - "$target" "$q" "$report" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
target, marker, report = sys.argv[1:4]
payload = {
  "schema_version": 1,
  "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
  "status": "failed",
  "target_manifest": target,
  "quarantine_marker": marker,
}
schema = json.loads(Path("ops/_schemas/datasets/corruption-drill-report.schema.json").read_text(encoding="utf-8"))
for key in schema.get("required", []):
  if key not in payload:
    raise SystemExit(f"corruption report missing required key: {key}")
open(report, "w", encoding="utf-8").write(json.dumps(payload, indent=2) + "\n")
PY
echo "corruption drill failed: target still valid json" >&2
exit 1
