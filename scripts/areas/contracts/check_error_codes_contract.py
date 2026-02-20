#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
error_codes = json.loads((ROOT / "docs" / "contracts" / "ERROR_CODES.json").read_text())["codes"]
status_map = json.loads((ROOT / "docs" / "contracts" / "ERROR_STATUS_MAP.json").read_text())["mappings"]
errors_doc = (ROOT / "docs" / "contracts" / "errors.md").read_text()

openapi = json.loads((ROOT / "configs" / "openapi" / "v1" / "openapi.snapshot.json").read_text())
openapi_codes = openapi["components"]["schemas"]["ApiErrorCode"]["enum"]
if sorted(error_codes) != sorted(openapi_codes):
    print("OpenAPI error code enum drift", file=sys.stderr)
    sys.exit(1)

rust_generated = (ROOT / "crates" / "bijux-atlas-api" / "src" / "generated" / "error_codes.rs").read_text()
for code in error_codes:
    if f'"{code}"' not in rust_generated:
        print(f"missing generated code in rust constants: {code}", file=sys.stderr)
        sys.exit(1)
    if f"### `{code}`" not in errors_doc:
        print(f"missing docs entry in docs/contracts/errors.md: {code}", file=sys.stderr)
        sys.exit(1)
    statuses = status_map.get(code)
    if not statuses:
        print(f"missing HTTP status mapping for code: {code}", file=sys.stderr)
        sys.exit(1)

print("error codes contract check passed")
