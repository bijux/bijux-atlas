#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
BASELINE="$ROOT/e2e/realdata/snapshots/release110_snapshot.json"
OUT="${ATLAS_REALDATA_SNAPSHOT_OUT:-$ROOT/artifacts/e2e/realdata/release110_snapshot.generated.json}"

"$ROOT/e2e/realdata/generate_snapshots.sh" "$OUT"

if [ "${ATLAS_REALDATA_UPDATE_SNAPSHOT:-0}" = "1" ]; then
  cp "$OUT" "$BASELINE"
  echo "updated baseline snapshot: $BASELINE"
  exit 0
fi

if [ "${ATLAS_REALDATA_ALLOW_BOOTSTRAP:-1}" = "1" ]; then
  if python3 - "$BASELINE" <<'PY'
import json,sys
data=json.load(open(sys.argv[1]))
raise SystemExit(0 if not data.get("entries") else 1)
PY
  then
    cp "$OUT" "$BASELINE"
    echo "bootstrapped baseline snapshot: $BASELINE"
    exit 0
  fi
fi

if ! diff -u "$BASELINE" "$OUT"; then
  echo "realdata snapshot drift detected" >&2
  exit 1
fi

echo "realdata snapshots verified"
