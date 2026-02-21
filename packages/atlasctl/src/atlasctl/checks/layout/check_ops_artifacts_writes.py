#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
allowlist_path = ROOT / "configs" / "ops" / "artifacts-allowlist.txt"
schema_path = ROOT / "ops" / "_schemas" / "meta" / "artifact-allowlist.schema.json"
allowlist = [
    line.strip()
    for line in allowlist_path.read_text(encoding="utf-8").splitlines()
    if line.strip() and not line.strip().startswith("#")
]
schema = json.loads(schema_path.read_text(encoding="utf-8"))
pat_entry = re.compile(schema["properties"]["entries"]["items"]["pattern"])
for entry in allowlist:
    if pat_entry.match(entry) is None:
        print(f"invalid allowlist entry (schema mismatch): {entry}", file=sys.stderr)
        raise SystemExit(1)

pat = re.compile(r"(?:^|[\"'\s=])(?:\./)?artifacts/[A-Za-z0-9_./-]+")
violations: list[str] = []

for path in sorted((ROOT / "ops").rglob("*")):
    if path.is_dir() or path.suffix not in {".sh", ".py"}:
        continue
    rel = path.relative_to(ROOT).as_posix()
    text = path.read_text(encoding="utf-8", errors="ignore")
    for i, line in enumerate(text.splitlines(), start=1):
        for m in pat.findall(line):
            raw = m.strip('"\' =')
            if raw.startswith("./"):
                raw = raw[2:]
            if raw.startswith("artifacts/ops"):
                continue
            if raw.startswith("artifacts/evidence"):
                continue
            allowed = raw in allowlist or rel in allowlist
            if not allowed:
                for item in allowlist:
                    if item.endswith("/") and raw.startswith(item):
                        allowed = True
                        break
            if allowed:
                continue
            violations.append(f"{rel}:{i}: forbidden artifact path `{raw}`")

if violations:
    print("ops artifact write policy failed:", file=sys.stderr)
    for v in violations:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("ops artifact write policy passed")
