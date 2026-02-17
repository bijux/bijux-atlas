#!/usr/bin/env python3
import json
from pathlib import Path

manifest = json.loads(Path("ops/e2e/k8s/tests/manifest.json").read_text())
ownership = json.loads(Path("ops/e2e/k8s/tests/ownership.json").read_text())

tests = manifest["tests"]
owners = ownership["owners"]

errors = []
all_scripts = {t["script"] for t in tests}

for t in tests:
    if "owner" not in t:
        errors.append(f"missing owner in manifest: {t['script']}")
    if "timeout_seconds" not in t:
        errors.append(f"missing timeout_seconds in manifest: {t['script']}")

claimed = set()
for owner, scripts in owners.items():
    for s in scripts:
        claimed.add(s)
        if s not in all_scripts:
            errors.append(f"ownership map has unknown test '{s}' for owner '{owner}'")

for s in sorted(all_scripts):
    if s not in claimed:
        errors.append(f"manifest test not in ownership map: {s}")

for t in tests:
    if t["owner"] not in owners:
        errors.append(f"manifest owner '{t['owner']}' not declared in ownership map for {t['script']}")

if errors:
    for e in errors:
        print(e)
    raise SystemExit(1)

print("k8s test contract check passed")
