#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from os import walk
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
OPS = ROOT / "ops"
errors: list[str] = []

# 1) observability pack owner is ops/obs/pack only.
for p in OPS.rglob("pack"):
    if not p.is_dir():
        continue
    rel = p.relative_to(ROOT).as_posix()
    if rel != "ops/obs/pack":
        errors.append(f"forbidden pack owner location: {rel}")

# 2) e2e/k8s/tests is composition-only wrappers.
allowed_e2e_k8s_test_files = {"run_all.sh", "report.sh", "pod-churn.sh"}
for p in (OPS / "e2e" / "k8s" / "tests").glob("*"):
    if p.is_file() and p.name not in allowed_e2e_k8s_test_files:
        errors.append(f"forbidden non-wrapper file in ops/e2e/k8s/tests: {p.name}")

# 3) no faults directory outside ops/stack/faults.
for p in OPS.rglob("faults"):
    if p.is_dir() and p.relative_to(ROOT).as_posix() != "ops/stack/faults":
        errors.append(f"forbidden faults dir outside stack: {p.relative_to(ROOT).as_posix()}")

# 4) use canonical fault API only.
blocked = re.compile(
    r"ops/stack/faults/(block-minio|toxiproxy-latency|throttle-network|cpu-throttle|fill-node-disk)\.sh"
)
skip_dirs = {
    ".git",
    "artifacts",
    "target",
    ".venv",
    ".mypy_cache",
    ".ruff_cache",
    "__pycache__",
}
skip_prefixes = {
    "ops/_generated/",
    "ops/_artifacts/",
}

for dirpath, dirnames, filenames in walk(ROOT):
    rel_dir = Path(dirpath).relative_to(ROOT).as_posix()
    if rel_dir != ".":
        rel_dir = f"{rel_dir}/"
    # Prune expensive/generated trees early.
    dirnames[:] = [
        d
        for d in dirnames
        if d not in skip_dirs and f"{rel_dir}{d}/" not in skip_prefixes
    ]
    for name in filenames:
        p = Path(dirpath) / name
        rel = p.relative_to(ROOT).as_posix()
        if any(rel.startswith(prefix) for prefix in skip_prefixes):
            continue
        if p.suffix not in {".sh", ".mk", ".py"}:
            continue
        if rel.startswith("ops/stack/faults/"):
            continue
        text = p.read_text(encoding="utf-8", errors="ignore")
        if blocked.search(text):
            errors.append(f"direct fault script reference outside canonical API: {rel}")

if errors:
    print("ops concept ownership contract failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("ops concept ownership contract passed")
