#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = [ROOT / "docs" / "operations", ROOT / "docs" / "quickstart", ROOT / "docs" / "development"]
EXCEPTIONS = ROOT / "configs/ops/public-surface-doc-exceptions.txt"
SURFACE = ROOT / "configs/ops/public-surface.json"


def load_surface() -> dict:
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def load_exceptions() -> set[str]:
    if not EXCEPTIONS.exists():
        return set()
    out = set()
    for line in EXCEPTIONS.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            out.add(line)
    return out


def main() -> int:
    surface = load_surface()
    public_targets = set(surface["make_targets"])
    errs: list[str] = []
    exceptions = load_exceptions()

    make_re = re.compile(r"\bmake\s+([a-zA-Z0-9_.-]+)")
    ops_script_re = re.compile(r"\./(ops/[^\s`]+(?:\.sh|\.py))")

    for base in DOCS:
        if not base.exists():
            continue
        for md in base.rglob("*.md"):
            text = md.read_text(encoding="utf-8", errors="ignore")
            rel = md.relative_to(ROOT).as_posix()
            for m in make_re.findall(text):
                if m == "ops-":
                    continue
                if not (m.startswith("ops-") or m in {"root", "root-local", "gates", "explain", "help"}):
                    continue
                key = f"{rel}::make {m}"
                if m not in public_targets and key not in exceptions:
                    errs.append(f"{rel}: non-public make target referenced: {m}")
            for script in ops_script_re.findall(text):
                key = f"{rel}::./{script}"
                if script.startswith("ops/run/"):
                    continue
                if key not in exceptions:
                    errs.append(f"{rel}: non-public ops script referenced: ./{script}")

    if errs:
        print("public surface docs check failed")
        for e in errs:
            print(f"- {e}")
        print(f"Use exceptions file for intentional transitions: {EXCEPTIONS.relative_to(ROOT)}")
        return 1

    print("public surface docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
