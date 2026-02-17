#!/usr/bin/env python3
# Purpose: generate docs/_generated/openapi artifacts from ops/openapi SSOT.
# Inputs: ops/openapi/v1/openapi.generated.json and openapi.snapshot.json.
# Outputs: docs/_generated/openapi/* generated files.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
src_dir = ROOT / "ops" / "openapi" / "v1"
out_dir = ROOT / "docs" / "_generated" / "openapi"
out_dir.mkdir(parents=True, exist_ok=True)

generated = src_dir / "openapi.generated.json"
snapshot = src_dir / "openapi.snapshot.json"
if not generated.exists():
    raise SystemExit(f"missing {generated}")
if not snapshot.exists():
    raise SystemExit(f"missing {snapshot}")

spec = json.loads(generated.read_text())
paths = sorted(spec.get("paths", {}).keys())

(out_dir / "openapi.generated.json").write_text(generated.read_text())
(out_dir / "openapi.snapshot.json").write_text(snapshot.read_text())

index = [
    "# OpenAPI Artifacts",
    "",
    "Generated from `ops/openapi/v1/`.",
    "",
    "- Canonical source: `ops/openapi/v1/openapi.generated.json`",
    "- Snapshot: `ops/openapi/v1/openapi.snapshot.json`",
    "",
    "## Paths",
    "",
]
for p in paths:
    index.append(f"- `{p}`")

(out_dir / "INDEX.md").write_text("\n".join(index) + "\n")
print("generated docs/_generated/openapi")
