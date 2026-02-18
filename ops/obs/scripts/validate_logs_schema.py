#!/usr/bin/env python3
# Purpose: validate JSON log lines against exported fields contract.
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCHEMA = json.loads((ROOT / "ops/obs/contract/logs-fields-contract.json").read_text())
REQUIRED = SCHEMA.get("required", [])
REQUIRED_FIELDS = SCHEMA.get("required_fields", REQUIRED)
ALLOWED_ENUM_VALUES = SCHEMA.get("allowed_enum_values", {})
PII_PROHIBITED = set(SCHEMA.get("pii_prohibited_fields", []))
EVENT_REGISTRY = SCHEMA.get("event_registry", {})
ALIASES = {
    "msg": {"msg", "message", "fields.message"},
    "ts": {"ts", "timestamp"},
    "request_id": {"request_id", "fields.request_id"},
}


def collect_keys(lines: list[str]) -> set[str]:
    seen: set[str] = set()
    for line in lines:
        if not line.startswith("{"):
            continue
        obj = json.loads(line)
        seen.update(obj.keys())
        if isinstance(obj.get("fields"), dict):
            for fk in obj["fields"].keys():
                seen.add(f"fields.{fk}")
    return seen


def find_value(obj: dict, key: str):
    if key in obj:
        return obj[key]
    fields = obj.get("fields")
    if isinstance(fields, dict):
        return fields.get(key)
    return None


def validate_line_objects(lines: list[str]) -> int:
    seen_events: set[str] = set()
    for idx, line in enumerate(lines, start=1):
        if not line.startswith("{"):
            continue
        obj = json.loads(line)
        # PII prohibited fields must never be present top-level or nested under fields.
        top_keys = set(obj.keys())
        fields = obj.get("fields")
        field_keys = set(fields.keys()) if isinstance(fields, dict) else set()
        bad_pii = sorted((top_keys | field_keys).intersection(PII_PROHIBITED))
        if bad_pii:
            print(f"line {idx}: prohibited PII fields present: {', '.join(bad_pii)}", file=sys.stderr)
            return 1
        # Enum checks when values are present.
        for enum_key, allowed in ALLOWED_ENUM_VALUES.items():
            value = find_value(obj, enum_key)
            if value is None:
                continue
            if value not in allowed:
                print(
                    f"line {idx}: invalid {enum_key}='{value}', allowed={allowed}",
                    file=sys.stderr,
                )
                return 1
        event_name = find_value(obj, "event_name")
        if isinstance(event_name, str):
            seen_events.add(event_name)
            reg = EVENT_REGISTRY.get(event_name, {})
            for req in reg.get("required_fields", []):
                if find_value(obj, req) is None:
                    print(f"line {idx}: event {event_name} missing required field: {req}", file=sys.stderr)
                    return 1

    # Request lifecycle events must exist in the observed corpus.
    for required_event in ("request_start", "request_end"):
        if required_event not in seen_events:
            print(f"missing required request lifecycle event: {required_event}", file=sys.stderr)
            return 1
    return 0


def validate_seen_keys(seen_keys: set[str]) -> int:
    for key in REQUIRED_FIELDS:
        if key in seen_keys:
            continue
        if any(alias in seen_keys for alias in ALIASES.get(key, set())):
            continue
        print(f"missing required log field: {key}", file=sys.stderr)
        return 1
    print("log fields contract passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--namespace", default="atlas-e2e")
    parser.add_argument("--release", default="atlas-e2e")
    parser.add_argument("--file", default="")
    parser.add_argument("--strict-live", action="store_true")
    args = parser.parse_args()

    if args.file:
        lines = Path(args.file).read_text(encoding="utf-8").splitlines()
        key_status = validate_seen_keys(collect_keys(lines))
        if key_status != 0:
            return key_status
        return validate_line_objects(lines)

    deploy_name = f"{args.release}-bijux-atlas"
    namespace = args.namespace
    cmd = ["kubectl", "-n", namespace, "logs", f"deploy/{deploy_name}", "--tail=200"]
    try:
        out = subprocess.check_output(cmd, text=True)
    except Exception as exc:  # pragma: no cover - integration path
        try:
            discovered_ns = subprocess.check_output(
                [
                    "kubectl",
                    "get",
                    "deploy",
                    "-A",
                    "-o",
                    "jsonpath={.items[?(@.metadata.name==\""
                    + deploy_name
                    + "\")].metadata.namespace}",
                ],
                text=True,
            ).strip()
            if discovered_ns:
                out = subprocess.check_output(
                    [
                        "kubectl",
                        "-n",
                        discovered_ns,
                        "logs",
                        f"deploy/{deploy_name}",
                        "--tail=200",
                    ],
                    text=True,
                )
                lines = out.splitlines()
                key_status = validate_seen_keys(collect_keys(lines))
                if key_status != 0:
                    return key_status
                return validate_line_objects(lines)
        except Exception:
            pass
        if args.strict_live:
            print(f"log schema check failed: {exc}", file=sys.stderr)
            return 1
        print(f"log schema check skipped: {exc}")
        return 0

    lines = out.splitlines()
    key_status = validate_seen_keys(collect_keys(lines))
    if key_status != 0:
        return key_status
    return validate_line_objects(lines)


if __name__ == "__main__":
    raise SystemExit(main())
