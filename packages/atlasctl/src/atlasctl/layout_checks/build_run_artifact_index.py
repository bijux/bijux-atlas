#!/usr/bin/env python3
# Purpose: generate per-run artifact index markdown page.
# Inputs: ops/_artifacts/<run_id> directory.
# Outputs: ops/_artifacts/<run_id>/index.md.
from __future__ import annotations
import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

p = argparse.ArgumentParser()
p.add_argument("--run-id", required=True)
args = p.parse_args()
run_dir = ROOT / "ops" / "_artifacts" / args.run_id
run_dir.mkdir(parents=True, exist_ok=True)
files = sorted(x for x in run_dir.rglob("*") if x.is_file())
lines = [f"# Artifact Index: {args.run_id}", "", f"- Run ID: `{args.run_id}`", "", "## Files", ""]
for f in files:
    lines.append(f"- `{f.relative_to(ROOT).as_posix()}`")
(run_dir / "index.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print(run_dir / "index.md")
