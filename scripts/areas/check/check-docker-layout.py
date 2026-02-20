#!/usr/bin/env python3
# Purpose: enforce Dockerfile layout policy (root shim + namespaced docker files).
# Inputs: repository file tree and symlink state.
# Outputs: non-zero on invalid Dockerfile placement/layout.
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
root_df = ROOT / "Dockerfile"
canon = ROOT / "docker" / "images" / "runtime" / "Dockerfile"

errors: list[str] = []
for required in [
    ROOT / "docker" / "images",
    ROOT / "docker" / "contracts",
    ROOT / "docker" / "scripts",
]:
    if not required.is_dir():
        errors.append(f"missing required docker directory: {required.relative_to(ROOT)}")

if not root_df.is_symlink():
    errors.append("root Dockerfile must be a symlink to docker/images/runtime/Dockerfile")
else:
    target = root_df.resolve()
    if target != canon.resolve():
        errors.append(f"root Dockerfile symlink target drift: expected {canon}, got {target}")

for p in ROOT.rglob("Dockerfile*"):
    rel = p.relative_to(ROOT).as_posix()
    if rel == "Dockerfile" or rel.startswith("docker/"):
        continue
    if "/artifacts/" in rel or rel.startswith("artifacts/"):
        continue
    errors.append(f"Dockerfile outside docker/ namespace forbidden: {rel}")

if errors:
    print("docker layout check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("docker layout check passed")
