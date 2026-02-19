#!/usr/bin/env python3
# Purpose: enforce root-defined public targets and block new non-internal targets outside root/help.
from __future__ import annotations

import re
import sys
from pathlib import Path

from public_make_targets import public_names

ROOT = Path(__file__).resolve().parents[2]
MAKEFILES = ROOT / "makefiles"
LEGACY = ROOT / "configs" / "ops" / "nonroot-legacy-targets.txt"
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", flags=re.M)


def parse_targets(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    return {t for t in TARGET_RE.findall(text) if not t.startswith(".")}


def load_legacy() -> set[str]:
    if not LEGACY.exists():
        return set()
    return {
        line.strip()
        for line in LEGACY.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def main() -> int:
    public = set(public_names())
    legacy = load_legacy()
    errs: list[str] = []

    by_file: dict[str, set[str]] = {}
    for mk in sorted(MAKEFILES.glob("*.mk")):
        by_file[mk.name] = parse_targets(mk)

    root_targets = by_file.get("root.mk", set())
    for target in sorted(public):
        if target not in root_targets:
            errs.append(f"public target must be defined in makefiles/root.mk: {target}")

    current_legacy: set[str] = set()
    for mk_name, targets in by_file.items():
        if mk_name in {"root.mk", "env.mk", "_macros.mk", "registry.mk", "help.mk"}:
            continue
        for target in sorted(targets):
            if target in public:
                errs.append(f"public target must not be defined outside root/help: makefiles/{mk_name}: {target}")
                continue
            if target.startswith("internal/") or target.startswith("_"):
                continue
            current_legacy.add(f"{mk_name}:{target}")

    for item in sorted(current_legacy - legacy):
        errs.append(
            f"new non-internal target outside root/help: {item} (must be internal/... or _..., or update baseline intentionally)"
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
