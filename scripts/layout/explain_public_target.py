#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"


def load_surface() -> dict:
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def lane_for_target(target: str) -> str:
    if target in {"root", "root-local", "root-local-fast", "root-local-summary"}:
        return "root-local-orchestration"
    if target.startswith("ops-"):
        return "ops"
    if target.startswith("ci-") or target in {"ci", "nightly"}:
        return "ci"
    if target in {"fmt", "lint", "test", "audit"}:
        return "rust"
    if target.startswith("docs"):
        return "docs"
    return "general"


def dry_run_expansion(target: str) -> list[str]:
    proc = subprocess.run(
        ["make", "-n", target],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return [f"(unable to expand: {proc.stderr.strip()})"]
    lines = [ln for ln in proc.stdout.splitlines() if ln.strip()]
    return lines[:20]


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("target")
    args = p.parse_args()
    s = load_surface()
    t = args.target
    if t in s["make_targets"]:
        print(f"public target: {t}")
        print(f"lane: {lane_for_target(t)}")
        print("dry-run expansion (first 20 lines):")
        for line in dry_run_expansion(t):
            print(f"  {line}")
        return 0
    print(f"not public: {t}")
    print("See: configs/ops/public-surface.json")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
