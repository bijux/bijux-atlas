#!/usr/bin/env python3
# Purpose: enforce script location policy for repository automation entrypoints.
# Inputs: git tracked *.sh/*.py paths.
# Outputs: non-zero exit when script placement violates policy.
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ALLOWED_PREFIXES = (
    "scripts/",
    "ops/",
)
ALLOWED_OPS_MARKERS = (
    "/scripts/",
    "/tests/",
    "/ci/",
    "/_lib/",
    "/run/",
    "/_lint/",
    "/runner/",
)
ALLOWED_OPS_PREFIXES = (
    "ops/_meta/",
    "ops/e2e/realdata/",
    "ops/load/reports/",
    "ops/stack/kind/",
    "ops/stack/minio/",
    "ops/stack/registry/",
    "ops/stack/toxiproxy/",
    "ops/stack/faults/",
    "ops/report/",
    "ops/e2e/smoke/",
    "ops/stack/scripts/",
)


def tracked_script_files() -> list[str]:
    out = subprocess.check_output(["git", "ls-files", "*.sh", "*.py"], cwd=ROOT, text=True)
    return [line.strip() for line in out.splitlines() if line.strip()]


def main() -> int:
    errors: list[str] = []
    for rel in tracked_script_files():
        if rel.startswith("scripts/"):
            continue
        if rel.startswith("ops/"):
            if any(marker in rel for marker in ALLOWED_OPS_MARKERS):
                continue
            if any(rel.startswith(prefix) for prefix in ALLOWED_OPS_PREFIXES):
                continue
            errors.append(
                f"{rel}: ops script path is outside approved automation zones"
            )
            continue
        errors.append(f"{rel}: scripts must live under scripts/ or ops/")
    if errors:
        print("script location check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("script location check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
