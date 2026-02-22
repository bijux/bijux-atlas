#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
image_line = re.compile(r"^\s*image\s*:\s*['\"]?([^'\"\s]+)")
errors: list[str] = []

scopes = [
    ROOT / "ops/load/compose",
    ROOT / "ops/obs/pack",
    ROOT / "ops/stack",
]

for scope in scopes:
    if not scope.exists():
        continue
    for path in sorted(scope.rglob("*.y*ml")):
        rel = path.relative_to(ROOT).as_posix()
        if rel.startswith("ops/_artifacts/") or rel.startswith("ops/_generated/"):
            continue
        for i, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            m = image_line.match(line)
            if not m:
                continue
            image = m.group(1)
            if "{{" in image:
                continue
            if image.endswith(":latest"):
                errors.append(f"{rel}:{i}: forbidden floating tag latest: {image}")
            if "@sha256:" not in image and ":" not in image.split("/")[-1]:
                errors.append(f"{rel}:{i}: image must pin tag or digest: {image}")

if errors:
    for e in errors:
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("image pin policy passed")
