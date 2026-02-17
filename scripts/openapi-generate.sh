#!/usr/bin/env sh
set -eu

./scripts/contracts/format_contracts.py
./scripts/contracts/generate_contract_artifacts.py
mkdir -p openapi/v1
cargo run --quiet -p bijux-atlas-api --bin atlas-openapi -- --out openapi/v1/openapi.generated.json
python3 - <<'PY'
import json
from pathlib import Path

contract = json.loads(Path("docs/contracts/ENDPOINTS.json").read_text())
expected = {e["path"] for e in contract["endpoints"]}
generated = json.loads(Path("openapi/v1/openapi.generated.json").read_text())
paths = set(generated.get("paths", {}).keys())
if expected != paths:
    missing = sorted(paths - expected)
    extra = sorted(expected - paths)
    raise SystemExit(
        f"openapi generation drift against ENDPOINTS.json; missing={missing} extra={extra}"
    )
print("openapi generation matches endpoint SSOT")
PY
