#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
AREAS = ("datasets", "e2e", "fixtures", "k8s", "load", "obs", "report", "run", "stack")
PATTERN = re.compile(r"ops/(" + "|".join(AREAS) + r")/scripts/[a-zA-Z0-9_./-]+\.sh")
ALLOWLIST = ROOT / "ops/_meta/cross-area-script-refs-allowlist.json"


def load_allowlist() -> set[str]:
    if not ALLOWLIST.exists():
        return set()
    payload = json.loads(ALLOWLIST.read_text(encoding="utf-8"))
    rows = payload.get("entries", []) if isinstance(payload, dict) else []
    items: set[str] = set()
    for row in rows:
        if not isinstance(row, dict):
            continue
        src = str(row.get("source_path", "")).strip()
        ref = str(row.get("referenced_path", "")).strip()
        if not src or not ref:
            continue
        items.add(f"{src}|{ref}")
    return items


def main() -> int:
    allowed = load_allowlist()
    seen: set[str] = set()
    errors: list[str] = []

    scan_roots = ("ops", "makefiles", "scripts", "docs", "configs")
    for root_name in scan_roots:
        base = ROOT / root_name
        if not base.exists():
            continue
        for file_path in base.rglob("*"):
            if not file_path.is_file():
                continue
            rel = file_path.relative_to(ROOT)
            if rel.name == "cross-area-script-refs-allowlist.json":
                continue
            if file_path.suffix in {".png", ".jpg", ".jpeg", ".gif", ".ico", ".pdf", ".svg", ".sqlite", ".gz", ".tgz", ".tar"}:
                continue
            text = file_path.read_text(encoding="utf-8", errors="ignore")
            for match in PATTERN.finditer(text):
                area = match.group(1)
                if len(rel.parts) > 1 and rel.parts[0] == "ops" and rel.parts[1] == area:
                    continue
                key = f"{rel}|{match.group(0)}"
                seen.add(key)
                if key not in allowed:
                    errors.append(key)

    stale = sorted(allowed - seen)
    if stale:
        print("cross-area script refs allowlist has stale entries:", file=sys.stderr)
        for item in stale:
            print(f"- {item}", file=sys.stderr)
        return 1

    if errors:
        print("new cross-area ops script references detected (use ops/run wrappers or update allowlist intentionally):", file=sys.stderr)
        for item in sorted(errors):
            print(f"- {item}", file=sys.stderr)
        return 1

    print("cross-area ops script reference check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
