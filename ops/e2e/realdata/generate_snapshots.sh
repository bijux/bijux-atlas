#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
QUERIES_JSON="$ROOT/ops/e2e/realdata/canonical_queries.json"
OUT_JSON="${1:-$ROOT/artifacts/ops/e2e/realdata/release110_snapshot.generated.json}"
mkdir -p "$(dirname "$OUT_JSON")"

python3 - "$BASE_URL" "$QUERIES_JSON" "$OUT_JSON" <<'PY'
import hashlib
import json
import pathlib
import sys
import urllib.request

base, qpath, out = sys.argv[1], pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])
queries = json.loads(qpath.read_text())["queries"]
entries = []
for q in queries:
    url = base + q
    req = urllib.request.Request(url, method="GET")
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            body = r.read()
            status = r.status
    except Exception as e:
        body = str(e).encode()
        status = 599
    entries.append(
        {
            "query": q,
            "status": status,
            "sha256": hashlib.sha256(body).hexdigest(),
            "size": len(body),
        }
    )

payload = {
    "schema_version": 1,
    "generated_from": "ops/e2e/realdata/canonical_queries.json",
    "entries": entries,
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
print(out)
PY
