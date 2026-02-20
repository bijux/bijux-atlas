#!/usr/bin/env python3
# Purpose: generate OpenAPI from the canonical API source and validate endpoint contract alignment.
# Inputs: docs/contracts/ENDPOINTS.json and crates/bijux-atlas-api OpenAPI generator.
# Outputs: configs/openapi/v1/openapi.generated.json (deterministic) and non-zero on drift/mismatch.
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ENDPOINTS = ROOT / "docs" / "contracts" / "ENDPOINTS.json"
OUT = ROOT / "configs" / "openapi" / "v1" / "openapi.generated.json"


def main() -> int:
    subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "bijux-atlas-api",
            "--bin",
            "atlas-openapi",
            "--",
            "--out",
            str(OUT),
        ],
        cwd=ROOT,
        check=True,
    )
    contract = json.loads(ENDPOINTS.read_text())
    generated = json.loads(OUT.read_text())
    contract_paths = {e["path"] for e in contract["endpoints"]}
    generated_paths = set(generated.get("paths", {}).keys())
    if contract_paths != generated_paths:
        missing = sorted(contract_paths - generated_paths)
        extra = sorted(generated_paths - contract_paths)
        print(
            f"OpenAPI/ENDPOINTS drift detected: missing={missing} extra={extra}",
            file=sys.stderr,
        )
        return 1
    OUT.write_text(json.dumps(generated, separators=(",", ":"), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
