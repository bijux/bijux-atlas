#!/usr/bin/env python3
# owner: platform
# purpose: validate canonical config files, schemas, and required-fields policy.
# stability: public
# called-by: make config-validate, make ci-config-check
# Purpose: validate canonical config files, schemas, and required-fields policy.
# Inputs: configs/**/*.json and docs/contracts/POLICY_SCHEMA.json
# Outputs: non-zero exit on validation failure
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def check_policy_schema_required(schema: dict, errors: list[str], path: str) -> None:
    if schema.get("type") != "object":
        errors.append(f"{path}: top-level type must be object")
        return
    required = schema.get("required", [])
    props = schema.get("properties", {})
    if not required:
        errors.append(f"{path}: top-level required must be non-empty")
    missing = sorted(set(props.keys()) - set(required))
    if missing:
        errors.append(f"{path}: missing required fields (no implicit defaults): {missing}")
    if schema.get("additionalProperties", True):
        errors.append(f"{path}: top-level additionalProperties must be false")


def check_policy_json_against_schema(cfg: dict, schema: dict, errors: list[str]) -> None:
    props = set(schema.get("properties", {}).keys())
    required = set(schema.get("required", []))
    cfg_keys = set(cfg.keys())
    missing = sorted(required - cfg_keys)
    extra = sorted(cfg_keys - props)
    if missing:
        errors.append(f"configs/policy/policy.json: missing required keys: {missing}")
    if extra:
        errors.append(f"configs/policy/policy.json: unknown keys: {extra}")


def check_ops_env_schema(schema: dict, errors: list[str]) -> None:
    if "version" not in schema:
        errors.append("configs/ops/env.schema.json: missing `version`")
    vars_map = schema.get("variables")
    if not isinstance(vars_map, dict) or not vars_map:
        errors.append("configs/ops/env.schema.json: `variables` must be non-empty object")
        return
    for name, spec in vars_map.items():
        if not isinstance(spec, dict):
            errors.append(f"configs/ops/env.schema.json: variable `{name}` must be object")
            continue
        if "type" not in spec:
            errors.append(f"configs/ops/env.schema.json: variable `{name}` missing type")
        if "default" not in spec and "default_from" not in spec:
            errors.append(f"configs/ops/env.schema.json: variable `{name}` missing default/default_from")


def check_tool_versions(data: dict, errors: list[str]) -> None:
    required = {"kind", "k6", "helm", "kubectl"}
    missing = sorted(required - set(data.keys()))
    if missing:
        errors.append(f"configs/ops/tool-versions.json: missing keys {missing}")


def main() -> int:
    errors: list[str] = []

    policy_schema = load("configs/policy/policy.schema.json")
    policy_cfg = load("configs/policy/policy.json")
    contracts_policy_schema = load("docs/contracts/POLICY_SCHEMA.json")
    ops_env_schema = load("configs/ops/env.schema.json")
    tool_versions = load("configs/ops/tool-versions.json")
    _perf = load("configs/perf/thresholds.json")
    _slo = load("configs/slo/slo.json")

    check_policy_schema_required(policy_schema, errors, "configs/policy/policy.schema.json")
    check_policy_json_against_schema(policy_cfg, policy_schema, errors)
    check_ops_env_schema(ops_env_schema, errors)
    check_tool_versions(tool_versions, errors)
    if policy_schema != contracts_policy_schema:
        errors.append("POLICY_SCHEMA drift: configs/policy/policy.schema.json != docs/contracts/POLICY_SCHEMA.json")

    if errors:
        print("config validation failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("config validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
