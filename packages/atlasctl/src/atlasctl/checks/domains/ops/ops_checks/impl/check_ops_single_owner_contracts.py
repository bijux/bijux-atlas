#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
OPS = ROOT / "ops"

canonical_areas = ["stack", "k8s", "obs", "load", "datasets", "e2e", "report", "fixtures", "run"]
errors: list[str] = []

for area in canonical_areas:
    base = OPS / area
    owner = base / "OWNER.md"
    contract = base / "CONTRACT.md"
    if not owner.exists():
        errors.append(f"missing OWNER.md: ops/{area}/OWNER.md")
    if not contract.exists():
        errors.append(f"missing CONTRACT.md: ops/{area}/CONTRACT.md")
    elif "schema_version" not in contract.read_text(encoding="utf-8", errors="ignore"):
        errors.append(f"contract missing schema_version marker: ops/{area}/CONTRACT.md")

charts = OPS / "k8s" / "charts"
if charts.exists():
    for p in charts.rglob("*"):
        if p.is_symlink():
            errors.append(f"forbidden symlink in charts tree: {p.relative_to(ROOT).as_posix()}")

for p in OPS.rglob("*qc*"):
    if p.is_file() and p.suffix in {".py", ".sh"}:
        rel = p.relative_to(ROOT).as_posix()
        if not rel.startswith("ops/datasets/scripts/"):
            errors.append(f"QC script outside datasets owner: {rel}")

# Fixture generation ownership: fixture generation scripts belong under ops/fixtures.
for p in OPS.rglob("*.sh"):
    rel = p.relative_to(ROOT).as_posix()
    if "fixture" in p.name and "generate" in p.name:
        if not rel.startswith("ops/fixtures/"):
            errors.append(f"fixture generation script outside fixtures owner: {rel}")

refs = (
    (ROOT / "ops" / "README.md").read_text(encoding="utf-8", errors="ignore")
    + "\n"
    + (ROOT / "makefiles" / "ops.mk").read_text(encoding="utf-8", errors="ignore")
    + "\n"
    + (ROOT / "docs" / "operations" / "INDEX.md").read_text(encoding="utf-8", errors="ignore")
)
for p in OPS.iterdir():
    if not p.is_dir() or p.name.startswith("_"):
        continue
    if p.name in {"registry"}:
        continue
    if f"ops/{p.name}" not in refs and f"`{p.name}`" not in refs:
        errors.append(f"potential orphan ops directory: ops/{p.name}")

if errors:
    print("ops single-owner/contracts/orphan check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops single-owner/contracts/orphan check passed")
