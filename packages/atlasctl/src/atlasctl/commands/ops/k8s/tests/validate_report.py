#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    import argparse

    p = argparse.ArgumentParser(description="Validate k8s suite report payload")
    p.add_argument("--report", required=True)
    args = p.parse_args()

    payload = json.loads(Path(args.report).read_text(encoding="utf-8"))
    errors: list[str] = []
    for key in ("schema_version", "run_id", "suite_id", "total", "failed", "passed", "results"):
        if key not in payload:
            errors.append(f"missing key: {key}")
    if payload.get("schema_version") != 1:
        errors.append("schema_version must be 1")
    if not isinstance(payload.get("results"), list):
        errors.append("results must be an array")
    if errors:
        print("k8s suite report validation failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("k8s suite report validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
