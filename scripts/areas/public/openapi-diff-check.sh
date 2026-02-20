#!/usr/bin/env sh
# owner: contracts
# purpose: detect OpenAPI drift against committed contract snapshot.
# stability: public
# called-by: make openapi-drift, make ops-openapi-validate
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -euo pipefail

./bin/atlasctl --quiet contracts check --checks drift
./bin/atlasctl --quiet contracts generate --generators artifacts openapi
python3 - <<'PY'
import json
from pathlib import Path

contract = json.loads(Path("docs/contracts/ENDPOINTS.json").read_text())
expected = {e["path"] for e in contract["endpoints"]}
generated = json.loads(Path("configs/openapi/v1/openapi.generated.json").read_text())
paths = set(generated.get("paths", {}).keys())
if expected != paths:
    missing = sorted(paths - expected)
    extra = sorted(expected - paths)
    raise SystemExit(
        f"openapi generation drift against ENDPOINTS.json; missing={missing} extra={extra}"
    )
print("openapi generation matches endpoint SSOT")
PY
if ! diff -u configs/openapi/v1/openapi.snapshot.json configs/openapi/v1/openapi.generated.json; then
  echo "OpenAPI drift detected. Regenerate snapshot intentionally when API contract changes." >&2
  exit 1
fi
