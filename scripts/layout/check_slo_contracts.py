#!/usr/bin/env python3
# Purpose: validate SLO SSOT configs and ensure SLO SLIs reference real metrics/labels.
from __future__ import annotations

import json
import argparse
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]


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


def _resolve_ref(schema: dict[str, Any], root_schema: dict[str, Any]) -> dict[str, Any]:
    ref = schema.get("$ref")
    if not isinstance(ref, str):
        return schema
    ref_path = ROOT / ref
    if ref.startswith("#/"):
        node: Any = root_schema
        for token in ref[2:].split("/"):
            node = node[token]
        if not isinstance(node, dict):
            raise ValueError(f"unsupported inline ref target: {ref}")
        return node
    payload = json.loads(ref_path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"invalid ref schema at {ref}")
    return payload


def _validate(schema: dict[str, Any], data: Any, path: str, errors: list[str], root_schema: dict[str, Any]) -> None:
    schema = _resolve_ref(schema, root_schema)
    t = schema.get("type")
    if isinstance(t, str) and not _type_ok(data, t):
        errors.append(f"{path}: expected type {t}")
        return

    if "enum" in schema and data not in schema["enum"]:
        errors.append(f"{path}: value {data!r} not in enum")

    if "const" in schema and data != schema["const"]:
        errors.append(f"{path}: expected const value {schema['const']!r}")

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
        item_schema = schema.get("items")
        if isinstance(item_schema, dict):
            for i, item in enumerate(data):
                _validate(item_schema, item, f"{path}[{i}]", errors, root_schema)

    if isinstance(data, dict):
        props = schema.get("properties", {})
        req = schema.get("required", [])
        for key in req:
            if key not in data:
                errors.append(f"{path}: missing required key `{key}`")
        if schema.get("additionalProperties", True) is False:
            for key in data.keys():
                if key not in props:
                    errors.append(f"{path}: unexpected key `{key}`")
        for key, subschema in props.items():
            if key in data and isinstance(subschema, dict):
                _validate(subschema, data[key], f"{path}.{key}", errors, root_schema)


def _validate_classes(classes: dict[str, Any], errors: list[str]) -> None:
    if classes.get("schema_version") != 1:
        errors.append("configs/ops/slo/classes.json: schema_version must be 1")
    declared: set[str] = set()
    for i, cls in enumerate(classes.get("classes", [])):
        name = cls.get("name")
        if name in declared:
            errors.append(f"configs/ops/slo/classes.json: duplicate class `{name}` at classes[{i}]")
        declared.add(name)
        patterns = cls.get("endpoint_patterns", [])
        if not isinstance(patterns, list) or not patterns:
            errors.append(f"configs/ops/slo/classes.json: class `{name}` must define endpoint_patterns")


def _validate_metric_refs(slo: dict[str, Any], metrics_contract: dict[str, Any], errors: list[str]) -> None:
    specs = metrics_contract.get("required_metric_specs", {})
    dyn_labels = set(metrics_contract.get("allowed_dynamic_labels", []))
    forbidden = set(metrics_contract.get("forbidden_labels", []))

    sli_ids = set()
    for sli in slo.get("slis", []):
        sid = sli.get("id")
        metric = sli.get("metric")
        labels = sli.get("labels", {})
        sli_ids.add(sid)
        if metric not in specs:
            errors.append(f"configs/ops/slo/slo.v1.json: SLI `{sid}` references unknown metric `{metric}`")
            continue
        spec = specs[metric]
        allowed = set(spec.get("required_labels", [])) | dyn_labels
        for label in labels.keys():
            if label in forbidden:
                errors.append(f"configs/ops/slo/slo.v1.json: SLI `{sid}` uses forbidden label `{label}`")
            if label not in allowed:
                errors.append(f"configs/ops/slo/slo.v1.json: SLI `{sid}` label `{label}` not declared for metric `{metric}`")

    for i, obj in enumerate(slo.get("slos", [])):
        sid = obj.get("sli")
        if sid not in sli_ids:
            errors.append(f"configs/ops/slo/slo.v1.json: slos[{i}] references unknown SLI `{sid}`")


def _validate_slis_file(slis_doc: dict[str, Any], metrics_contract: dict[str, Any], errors: list[str]) -> None:
    specs = metrics_contract.get("required_metric_specs", {})
    dyn_labels = set(metrics_contract.get("allowed_dynamic_labels", []))
    forbidden = set(metrics_contract.get("forbidden_labels", []))
    extra_allowed = {"route", "status", "dataset", "reason", "stage", "source"}

    if slis_doc.get("schema_version") != 1:
        errors.append("configs/ops/slo/slis.v1.json: schema_version must be 1")

    for i, sli in enumerate(slis_doc.get("slis", [])):
        sid = sli.get("id", f"index-{i}")
        status = sli.get("status", "enforced")
        metric = sli.get("metric")
        if not isinstance(metric, str):
            errors.append(f"configs/ops/slo/slis.v1.json: sli `{sid}` metric must be string")
            continue
        if status == "enforced" and metric not in specs:
            errors.append(f"configs/ops/slo/slis.v1.json: enforced sli `{sid}` references unknown metric `{metric}`")

        sec = sli.get("secondary_metric")
        sec_status = sli.get("secondary_metric_status", "enforced")
        if status == "enforced" and isinstance(sec, str) and sec not in specs and sec_status != "planned":
            errors.append(f"configs/ops/slo/slis.v1.json: enforced sli `{sid}` secondary metric `{sec}` unknown")

        labels = sli.get("labels", {})
        if not isinstance(labels, dict):
            errors.append(f"configs/ops/slo/slis.v1.json: sli `{sid}` labels must be object")
            continue
        allowed = set(specs.get(metric, {}).get("required_labels", [])) | dyn_labels | extra_allowed
        for label in labels.keys():
            if label in forbidden:
                errors.append(f"configs/ops/slo/slis.v1.json: sli `{sid}` uses forbidden label `{label}`")
            if status == "enforced" and label not in allowed:
                errors.append(f"configs/ops/slo/slis.v1.json: sli `{sid}` label `{label}` not allowed for `{metric}`")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("all", "schema", "metrics", "slis"), default="all")
    args = parser.parse_args()

    errors: list[str] = []

    classes = json.loads((ROOT / "configs/ops/slo/classes.json").read_text(encoding="utf-8"))
    sli_schema = json.loads((ROOT / "configs/ops/slo/sli.schema.json").read_text(encoding="utf-8"))
    slo_schema = json.loads((ROOT / "configs/ops/slo/slo.schema.json").read_text(encoding="utf-8"))
    slo = json.loads((ROOT / "configs/ops/slo/slo.v1.json").read_text(encoding="utf-8"))
    slis_doc = json.loads((ROOT / "configs/ops/slo/slis.v1.json").read_text(encoding="utf-8"))
    metrics_contract = json.loads((ROOT / "ops/obs/contract/metrics-contract.json").read_text(encoding="utf-8"))

    if args.mode in {"all", "schema"}:
        _validate_classes(classes, errors)
        for i, sli in enumerate(slo.get("slis", [])):
            _validate(sli_schema, sli, f"configs/ops/slo/slo.v1.json.slis[{i}]", errors, sli_schema)
        _validate(slo_schema, slo, "configs/ops/slo/slo.v1.json", errors, slo_schema)

        refs = set(slo.get("change_policy", {}).get("references", []))
        for required_doc in {"docs/operations/slo/CHANGE_POLICY.md", "docs/operations/slo/CHANGELOG.md"}:
            if required_doc not in refs:
                errors.append(f"configs/ops/slo/slo.v1.json: change_policy.references missing `{required_doc}`")

    if args.mode in {"all", "metrics"}:
        _validate_metric_refs(slo, metrics_contract, errors)

    if args.mode in {"all", "slis"}:
        _validate_slis_file(slis_doc, metrics_contract, errors)

    if errors:
        print("slo contracts check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("slo contracts check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
