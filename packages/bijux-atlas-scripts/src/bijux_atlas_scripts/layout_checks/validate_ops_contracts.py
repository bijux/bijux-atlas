#!/usr/bin/env python3
# Purpose: validate canonical ops manifests against ops/_schemas contracts.
# Inputs: schema files under ops/_schemas and referenced manifest files.
# Outputs: non-zero exit on contract violations.
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]


def _type_ok(value: Any, t: str) -> bool:
    if t == "object":
        return isinstance(value, dict)
    if t == "array":
        return isinstance(value, list)
    if t == "string":
        return isinstance(value, str)
    if t == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if t == "number":
        return (isinstance(value, int) and not isinstance(value, bool)) or isinstance(value, float)
    if t == "boolean":
        return isinstance(value, bool)
    return True


def _validate(schema: dict[str, Any], data: Any, path: str, errors: list[str]) -> None:
    t = schema.get("type")
    if isinstance(t, str) and not _type_ok(data, t):
        errors.append(f"{path}: expected type {t}")
        return

    if "enum" in schema and data not in schema["enum"]:
        errors.append(f"{path}: value {data!r} not in enum")

    if isinstance(data, str):
        if "minLength" in schema and len(data) < int(schema["minLength"]):
            errors.append(f"{path}: minLength {schema['minLength']} violated")
        if "pattern" in schema and re.match(schema["pattern"], data) is None:
            errors.append(f"{path}: pattern {schema['pattern']} mismatch")

    if isinstance(data, (int, float)) and not isinstance(data, bool):
        if "minimum" in schema and data < schema["minimum"]:
            errors.append(f"{path}: minimum {schema['minimum']} violated")
        if "maximum" in schema and data > schema["maximum"]:
            errors.append(f"{path}: maximum {schema['maximum']} violated")

    if isinstance(data, list):
        if "minItems" in schema and len(data) < int(schema["minItems"]):
            errors.append(f"{path}: minItems {schema['minItems']} violated")
        items = schema.get("items")
        if isinstance(items, dict):
            for i, item in enumerate(data):
                _validate(items, item, f"{path}[{i}]", errors)

    if isinstance(data, dict):
        props = schema.get("properties", {})
        req = schema.get("required", [])
        for key in req:
            if key not in data:
                errors.append(f"{path}: missing required key `{key}`")
        addl = schema.get("additionalProperties", True)
        if addl is False:
            for key in data.keys():
                if key not in props:
                    errors.append(f"{path}: unexpected key `{key}`")
        for key, subschema in props.items():
            if key in data and isinstance(subschema, dict):
                _validate(subschema, data[key], f"{path}.{key}", errors)

        # `if/then` branches are enforced by domain validators where needed.


def validate_pair(schema_rel: str, data_rel: str, errors: list[str]) -> None:
    schema = json.loads((ROOT / schema_rel).read_text(encoding="utf-8"))
    data = json.loads((ROOT / data_rel).read_text(encoding="utf-8"))
    local_errors: list[str] = []
    _validate(schema, data, data_rel, local_errors)
    if local_errors:
        errors.extend(local_errors)


def main() -> int:
    errors: list[str] = []
    pairs = [
        ("ops/_schemas/stack/profile-manifest.schema.json", "ops/stack/profiles.json"),
        ("ops/_schemas/k8s/install-matrix.schema.json", "ops/k8s/install-matrix.json"),
        ("ops/_schemas/load/suite-manifest.schema.json", "ops/load/suites/suites.json"),
        ("ops/_schemas/obs/drill-manifest.schema.json", "ops/obs/drills/drills.json"),
        ("ops/_schemas/report/schema.json", "ops/_generated_committed/examples/report.example.json"),
        ("ops/_schemas/report/unified.schema.json", "ops/_generated_committed/examples/report.unified.example.json"),
        ("ops/_schemas/stack/version-manifest.schema.json", "ops/stack/version-manifest.json"),
        ("ops/_schemas/meta/ownership.schema.json", "ops/_meta/ownership.json"),
        ("ops/_schemas/meta/layer-contract.schema.json", "ops/_meta/layer-contract.json"),
        ("ops/_schemas/meta/namespaces.schema.json", "configs/ops/namespaces.json"),
        ("ops/_schemas/meta/ports.schema.json", "configs/ops/ports.json"),
        ("ops/_schemas/meta/pins.schema.json", "configs/ops/pins.json"),
        ("ops/_schemas/meta/budgets.schema.json", "configs/ops/budgets.json"),
        ("ops/_schemas/load/pinned-queries-lock.schema.json", "ops/load/queries/pinned-v1.lock"),
        ("ops/_schemas/load/perf-baseline.schema.json", "configs/ops/perf/baselines/local.json"),
        ("ops/_schemas/load/perf-baseline.schema.json", "configs/ops/perf/baselines/ci-runner.json"),
        ("ops/_schemas/datasets/manifest-lock.schema.json", "ops/datasets/manifest.lock"),
        ("ops/_schemas/e2e-scenarios.schema.json", "ops/e2e/scenarios/scenarios.json"),
        ("ops/_schemas/e2e-realdata-scenarios.schema.json", "ops/e2e/realdata/scenarios.json"),
        ("ops/_schemas/e2e-suites.schema.json", "ops/e2e/suites/suites.json"),
        ("ops/_schemas/obs/suites.schema.json", "ops/obs/suites/suites.json"),
        ("ops/_schemas/obs/budgets.schema.json", "configs/ops/obs/budgets.json"),
    ]
    for schema_rel, data_rel in pairs:
        validate_pair(schema_rel, data_rel, errors)

    # Validate artifact allowlist text contract using schema wrapper.
    allow_entries = [
        line.strip()
        for line in (ROOT / "configs/ops/artifacts-allowlist.txt").read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    ]
    schema = json.loads((ROOT / "ops/_schemas/meta/artifact-allowlist.schema.json").read_text(encoding="utf-8"))
    _validate(schema, {"entries": allow_entries}, "configs/ops/artifacts-allowlist.txt", errors)

    # Enforce generated versions.json from tool-versions SSOT.
    tool_versions = json.loads((ROOT / "configs/ops/tool-versions.json").read_text(encoding="utf-8"))
    stack_versions = json.loads((ROOT / "ops/stack/versions.json").read_text(encoding="utf-8"))
    if stack_versions != tool_versions:
        errors.append("ops/stack/versions.json must exactly match configs/ops/tool-versions.json")
    legacy_suite_schema = json.loads((ROOT / "ops/load/contracts/suite-schema.json").read_text(encoding="utf-8"))
    canonical_suite_schema = json.loads((ROOT / "ops/_schemas/load/suite-manifest.schema.json").read_text(encoding="utf-8"))
    if legacy_suite_schema != canonical_suite_schema:
        errors.append("ops/load/contracts/suite-schema.json must mirror ops/_schemas/load/suite-manifest.schema.json")

    layer_contract = json.loads((ROOT / "ops/_meta/layer-contract.json").read_text(encoding="utf-8"))
    namespaces_ssot = json.loads((ROOT / "configs/ops/namespaces.json").read_text(encoding="utf-8")).get("namespaces", {})
    ports_ssot = json.loads((ROOT / "configs/ops/ports.json").read_text(encoding="utf-8")).get("ports", {})
    if layer_contract.get("namespaces") != namespaces_ssot:
        errors.append("ops/_meta/layer-contract.json namespaces must match configs/ops/namespaces.json")
    if layer_contract.get("ports") != ports_ssot:
        errors.append("ops/_meta/layer-contract.json ports must match configs/ops/ports.json")
    profiles_path = layer_contract.get("ssot", {}).get("profiles")
    if not isinstance(profiles_path, str) or not (ROOT / profiles_path).exists():
        errors.append("ops/_meta/layer-contract.json ssot.profiles must point to existing profile manifest")

    if errors:
        print("ops contracts check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("ops contracts check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
