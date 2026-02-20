#!/usr/bin/env python3
# owner: platform
# purpose: enforce config drift policy between SSOT schemas/docs and generated config registry.
# stability: public
# called-by: make config-drift, make ci-config-check
# Purpose: enforce config drift policy between SSOT schemas/docs and generated config registry.
# Inputs: configs/config-key-registry.md, docs/contracts/CONFIG_KEYS.json, configs/policy/policy.schema.json, docs/contracts/POLICY_SCHEMA.json
# Outputs: non-zero on drift
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def main() -> int:
    errors: list[str] = []
    policy_schema = json.loads((ROOT / "configs/policy/policy.schema.json").read_text(encoding="utf-8"))
    contracts_policy = json.loads((ROOT / "docs/contracts/POLICY_SCHEMA.json").read_text(encoding="utf-8"))
    if policy_schema != contracts_policy:
        errors.append("policy schema drift between configs and docs/contracts")

    key_doc = ROOT / "configs/config-key-registry.md"
    if not key_doc.exists():
        errors.append("missing generated configs/config-key-registry.md (run make config-validate)")
    else:
        keys = json.loads((ROOT / "docs/contracts/CONFIG_KEYS.json").read_text(encoding="utf-8")).get("env_keys", [])
        text = key_doc.read_text(encoding="utf-8")
        for key in keys:
            if f"`{key}`" not in text:
                errors.append(f"config key registry missing `{key}`")
                break

    if errors:
        print("config drift check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("config drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
