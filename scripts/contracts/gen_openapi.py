#!/usr/bin/env python3
# Purpose: generate OpenAPI from the canonical API source and validate endpoint contract alignment.
# Inputs: docs/contracts/ENDPOINTS.json and crates/bijux-atlas-api OpenAPI generator.
# Outputs: configs/openapi/v1/openapi.generated.json (deterministic) and non-zero on drift/mismatch.
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ENDPOINTS = ROOT / "docs" / "contracts" / "ENDPOINTS.json"
FILTERS = ROOT / "docs" / "contracts" / "FILTERS.json"
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
    filters_contract = json.loads(FILTERS.read_text())
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
    generated = apply_filters_contract(generated, filters_contract)
    api_contract_version = contract.get("api_contract_version", "v1")
    info = generated.setdefault("info", {})
    info["x-api-contract-version"] = api_contract_version
    OUT.write_text(json.dumps(generated, separators=(",", ":"), sort_keys=True))
    return 0


def apply_filters_contract(generated: dict, filters_contract: dict) -> dict:
    path = filters_contract["endpoint"]
    paths = generated.get("paths", {})
    if path not in paths or "get" not in paths[path]:
        return generated
    params = paths[path]["get"].get("parameters", [])
    preserved = [p for p in params if p.get("in") != "query" or p.get("name") in {"release", "species", "assembly", "limit", "cursor", "include", "pretty", "explain"}]
    filter_params = []
    for f in filters_contract.get("filters", []):
        schema = {"type": f["type"] if f["type"] != "range" else "string"}
        if f["name"] == "range":
            schema["pattern"] = r"^[^:]+:[0-9]+-[0-9]+$"
        if f["type"] == "integer":
            schema["minimum"] = 0
        if f.get("max_length"):
            schema["maxLength"] = f["max_length"]
        filter_params.append({"name": f["name"], "in": "query", "schema": schema})
    paths[path]["get"]["parameters"] = preserved + filter_params
    return generated


if __name__ == "__main__":
    raise SystemExit(main())
