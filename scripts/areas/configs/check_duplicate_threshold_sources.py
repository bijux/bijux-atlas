#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCAN_ROOTS = (ROOT / "configs", ROOT / "ops")
SKIP_DIRS = {".git", ".hg", ".svn", "artifacts", "target", "node_modules", ".venv", "__pycache__"}


def _iter_candidates(pattern: str) -> list[Path]:
  out: list[Path] = []
  for scan_root in SCAN_ROOTS:
    if not scan_root.exists():
      continue
    for path in scan_root.rglob(pattern):
      rel_parts = path.relative_to(ROOT).parts
      if any(part in SKIP_DIRS for part in rel_parts):
        continue
      out.append(path)
  return out


def main() -> int:
  errs=[]
  expected = {
    "dataset_qc": "configs/ops/dataset-qc-thresholds.v1.json",
    "cache_budget": "configs/ops/cache-budget-thresholds.v1.json",
    "k6_thresholds": "configs/perf/k6-thresholds.v1.json",
  }
  mirrors = {
    "k6_thresholds": {"ops/load/contracts/k6-thresholds.v1.json"},
    "dataset_qc": set(),
    "cache_budget": set(),
  }
  patterns = {
    "dataset_qc": "*dataset*qc*threshold*.json",
    "cache_budget": "*cache*budget*threshold*.json",
    "k6_thresholds": "*k6*threshold*.json",
  }
  for key,pat in patterns.items():
    hits=[p.relative_to(ROOT).as_posix() for p in _iter_candidates(pat)]
    canonical=expected[key]
    for h in hits:
      if h != canonical and h not in mirrors.get(key, set()) and not h.startswith("ops/_generated") and not h.startswith("ops/_generated_committed"):
        errs.append(f"duplicate threshold source for {key}: {h} (canonical {canonical})")
  if errs:
    print("duplicate thresholds check failed")
    for e in errs:
      print(f"- {e}")
    return 1
  print("duplicate thresholds check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
