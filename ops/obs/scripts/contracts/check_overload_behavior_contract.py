#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "ops/obs/contract/overload-behavior-contract.json"
OPENAPI = ROOT / "configs/openapi/v1/openapi.generated.json"
API_ERRORS = ROOT / "crates/bijux-atlas-api/src/generated/error_codes.rs"


def main() -> int:
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    openapi = json.loads(OPENAPI.read_text(encoding="utf-8"))
    errors_rs = API_ERRORS.read_text(encoding="utf-8")

    heavy_required_any = {
        str(x) for x in contract["overload_response_contract"]["heavy_endpoints_must_include_any"]
    }
    cheap_required = {str(x) for x in contract["overload_response_contract"]["cheap_endpoints_must_include"]}
    errors: list[str] = []

    for path in contract.get("heavy_endpoints", []):
        responses = openapi.get("paths", {}).get(path, {}).get("get", {}).get("responses", {})
        if not (set(responses.keys()) & heavy_required_any):
            errors.append(
                f"heavy endpoint {path} must expose at least one overload status in: {', '.join(sorted(heavy_required_any))}"
            )

    for path in contract.get("cheap_endpoints", []):
        responses = openapi.get("paths", {}).get(path, {}).get("get", {}).get("responses", {})
        missing = sorted(cheap_required - set(responses.keys()))
        if missing:
            errors.append(f"cheap endpoint {path} missing required statuses: {', '.join(missing)}")

    for code in contract["overload_response_contract"].get("policy_error_codes", []):
        if f'"{code}"' not in errors_rs:
            errors.append(f"missing api error code in generated catalog: {code}")

    if errors:
        print("overload behavior contract failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("overload behavior contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
