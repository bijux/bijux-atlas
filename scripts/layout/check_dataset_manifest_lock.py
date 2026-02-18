#!/usr/bin/env python3
# Purpose: validate ops/datasets/manifest.lock against schema contract.
# Inputs: ops/_schemas/datasets/manifest-lock.schema.json and ops/datasets/manifest.lock.
# Outputs: non-zero exit on schema mismatch.
from __future__ import annotations
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
schema = json.loads((ROOT / "ops/_schemas/datasets/manifest-lock.schema.json").read_text(encoding="utf-8"))
data = json.loads((ROOT / "ops/datasets/manifest.lock").read_text(encoding="utf-8"))

errs: list[str] = []
for req in schema["required"]:
    if req not in data:
        errs.append(f"missing required key: {req}")

for i, entry in enumerate(data.get("entries", [])):
    if not isinstance(entry, dict):
        errs.append(f"entry[{i}] must be object")
        continue
    for req in ("name", "id", "checksums"):
        if req not in entry:
            errs.append(f"entry[{i}] missing {req}")
    if "id" in entry and re.match(r"^[0-9]+/[a-z_]+/[A-Za-z0-9]+$", str(entry["id"])) is None:
        errs.append(f"entry[{i}] invalid id")
    checksums = entry.get("checksums", {})
    if not isinstance(checksums, dict):
        errs.append(f"entry[{i}] checksums must be object")
    else:
        for k, v in checksums.items():
            if re.match(r"^[a-f0-9]{64}$", str(v)) is None:
                errs.append(f"entry[{i}] invalid checksum for {k}")

if errs:
    print("dataset manifest lock schema check failed:", file=sys.stderr)
    for e in errs:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("dataset manifest lock schema check passed")
