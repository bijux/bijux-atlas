#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
from pathlib import Path


def main() -> int:
    schema_path = Path("configs/policy/policy.schema.json")
    config_path = Path("configs/policy/policy.json")
    schema = json.loads(schema_path.read_text())
    config = json.loads(config_path.read_text())

    required = set(schema.get("required", []))
    config_keys = set(config.keys())
    if required != config_keys:
        missing = sorted(required - config_keys)
        extra = sorted(config_keys - required)
        raise SystemExit(
            f"policy schema drift: required/config keys mismatch; missing={missing} extra={extra}"
        )

    if schema.get("additionalProperties", True):
        raise SystemExit("policy schema drift: top-level additionalProperties must be false")

    canonical = json.dumps(schema, indent=2, sort_keys=True) + "\n"
    if schema_path.read_text() != canonical:
        raise SystemExit(
            "policy schema drift: schema file is not canonical (run formatter/regenerate)"
        )

    print("policy schema drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())