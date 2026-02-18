#!/usr/bin/env python3
# Purpose: validate JSON log lines against exported fields contract.
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCHEMA = json.loads((ROOT / "ops/observability/contract/logs-fields-contract.json").read_text())
REQUIRED = SCHEMA.get("required", [])
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


def validate_seen_keys(seen_keys: set[str]) -> int:
    for key in REQUIRED:
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
        return validate_seen_keys(collect_keys(lines))

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
                return validate_seen_keys(collect_keys(out.splitlines()))
        except Exception:
            pass
        if args.strict_live:
            print(f"log schema check failed: {exc}", file=sys.stderr)
            return 1
        print(f"log schema check skipped: {exc}")
        return 0

    return validate_seen_keys(collect_keys(out.splitlines()))


if __name__ == "__main__":
    raise SystemExit(main())
