#!/usr/bin/env python3
# Purpose: validate unified e2e scenario manifest and docs references.
# Inputs: ops/e2e/scenarios/scenarios.json, ops/_schemas/e2e-scenarios.schema.json, docs/operations/e2e/*.md.
# Outputs: non-zero on missing/invalid scenarios or docs references.
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
manifest = json.loads((ROOT / "ops/e2e/scenarios/scenarios.json").read_text(encoding="utf-8"))
schema = json.loads((ROOT / "ops/_schemas/e2e-scenarios.schema.json").read_text(encoding="utf-8"))

errors: list[str] = []
for key in schema.get("required", []):
    if key not in manifest:
        errors.append(f"missing required key: {key}")

scenarios = manifest.get("scenarios", [])
seen: set[str] = set()
for i, s in enumerate(scenarios):
    sid = s.get("id")
    if not isinstance(sid, str) or re.match(r"^[a-z0-9-]+$", sid) is None:
        errors.append(f"scenario[{i}] invalid id")
        continue
    if sid in seen:
        errors.append(f"duplicate scenario id: {sid}")
    seen.add(sid)
    compose = s.get("compose", {})
    for required in ("stack", "obs", "datasets", "load"):
        if required not in compose or not isinstance(compose.get(required), bool):
            errors.append(f"scenario `{sid}` missing boolean compose.{required}")

    entry = s.get("entrypoint", "")
    if not isinstance(entry, str) or not entry.startswith("make "):
        errors.append(f"scenario `{sid}` invalid entrypoint")
    else:
        target = entry.split()[1]
        if target not in (ROOT / "ops/_meta/surface.json").read_text(encoding="utf-8") and target not in (ROOT / "makefiles/ops.mk").read_text(encoding="utf-8"):
            errors.append(f"scenario `{sid}` entrypoint target not found: {target}")

# docs reference gate: scenario:<id> markers in docs/operations/e2e/*.md
refs: set[str] = set()
for doc in sorted((ROOT / "docs/operations/e2e").glob("*.md")):
    txt = doc.read_text(encoding="utf-8")
    for m in re.finditer(r"scenario:([a-z0-9-]+)", txt):
        refs.add(m.group(1))

for ref in sorted(refs):
    if ref not in seen:
        errors.append(f"docs reference unknown scenario: {ref}")

if errors:
    print("e2e scenario contract failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("e2e scenario contract passed")
