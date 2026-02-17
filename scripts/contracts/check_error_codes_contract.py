#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
error_codes = json.loads((ROOT / "docs" / "contracts" / "ERROR_CODES.json").read_text())["codes"]

openapi = json.loads((ROOT / "openapi" / "v1" / "openapi.snapshot.json").read_text())
openapi_codes = openapi["components"]["schemas"]["ApiErrorCode"]["enum"]
if sorted(error_codes) != sorted(openapi_codes):
    print("OpenAPI error code enum drift", file=sys.stderr)
    sys.exit(1)

rust_generated = (ROOT / "crates" / "bijux-atlas-api" / "src" / "generated" / "error_codes.rs").read_text()
for code in error_codes:
    if f'"{code}"' not in rust_generated:
        print(f"missing generated code in rust constants: {code}", file=sys.stderr)
        sys.exit(1)

print("error codes contract check passed")