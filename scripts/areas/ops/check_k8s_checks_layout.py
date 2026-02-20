#!/usr/bin/env python3
# Purpose: enforce k8s checks directory layout and group naming constraints.
# Inputs: ops/k8s/tests/checks tree and ops/k8s/tests/manifest.json.
# Outputs: non-zero exit on layout budget or naming contract violations.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CHECKS = ROOT / "ops" / "k8s" / "tests" / "checks"
MAX_FILES = 10


def main() -> int:
    errors: list[str] = []

    for area in sorted(p for p in CHECKS.iterdir() if p.is_dir() and p.name != "_lib"):
        direct_tests = sorted(area.glob("test_*.sh"))
        has_submodules = any(p.is_dir() for p in area.iterdir())
        if len(direct_tests) > MAX_FILES and not has_submodules:
            errors.append(
                f"{area.relative_to(ROOT)} has {len(direct_tests)} test files; max {MAX_FILES} without submodules"
            )

    cfg = CHECKS / "config"
    for f in sorted(cfg.glob("test_*.sh")):
        if "config" not in f.name and "envfrom" not in f.name:
            errors.append(f"{f.relative_to(ROOT)} is under config/ but does not look config-related")

    manifest = json.loads((ROOT / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    for t in manifest.get("tests", []):
        groups = {g for g in t.get("groups", []) if isinstance(g, str)}
        if "obs" in groups:
            errors.append(f"{t.get('script')}: ambiguous group `obs` forbidden; use `observability`")

    if errors:
        print("k8s checks layout lint failed")
        for e in errors:
            print(f"- {e}")
        return 1
    print("k8s checks layout lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
