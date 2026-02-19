#!/usr/bin/env python3
# Purpose: enforce public target publication in root.mk and non-root target namespaces.
# Inputs: makefiles/*.mk and configs/ops/public-surface.json.
# Outputs: exits non-zero on ownership/namespace violations.
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAKEFILES = ROOT / "makefiles"
SURFACE = ROOT / "configs/ops/public-surface.json"

TARGET_RE = re.compile(r"^([a-zA-Z0-9_.-]+):(?:\s|$)", flags=re.M)
ALLOWED_NON_ROOT_PREFIXES = (
    "_",
    "ci-",
    "ops-",
    "dev-",
    "layout-",
    "path-",
    "culprits-",
    "crate-",
    "cli-",
    "policy-",
    "docs",
    "help",
    "fmt",
    "lint",
    "check",
    "test",
    "coverage",
    "audit",
    "openapi-",
    "api-",
    "compat-",
    "fetch-",
    "load-",
    "perf-",
    "query-",
    "critical-",
    "cold-",
    "memory-",
    "run-",
    "bench-",
    "governance-",
    "e2e-",
    "observability-",
    "stack-",
    "ingest-",
)


def parse_targets(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    out = set(TARGET_RE.findall(text))
    return {t for t in out if not t.startswith(".")}


def main() -> int:
    surface = json.loads(SURFACE.read_text(encoding="utf-8"))
    public_targets = set(surface.get("make_targets", []))
    public_targets.discard("help")

    by_file: dict[str, set[str]] = {}
    for mk in sorted(MAKEFILES.glob("*.mk")):
        by_file[mk.name] = parse_targets(mk)

    errs: list[str] = []

    root_text = (MAKEFILES / "root.mk").read_text(encoding="utf-8")
    phony = set()
    for line in root_text.splitlines():
        if line.startswith(".PHONY:"):
            phony.update(line.replace(".PHONY:", "", 1).split())
    for target in sorted(public_targets):
        if target not in phony:
            errs.append(f"public target missing from makefiles/root.mk publication surface: {target}")

    for mk_name, targets in by_file.items():
        if mk_name == "root.mk":
            continue
        for target in sorted(targets):
            if target in public_targets:
                continue
            if target.startswith(ALLOWED_NON_ROOT_PREFIXES):
                continue
            errs.append(
                f"non-root target missing internal namespace in makefiles/{mk_name}: {target} "
                f"(expected prefix like internal- / ops- / ci- / _)"
            )

    if errs:
        print("makefile target boundaries check failed", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("makefile target boundaries check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
