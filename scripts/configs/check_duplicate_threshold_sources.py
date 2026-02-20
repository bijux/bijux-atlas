#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


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
    hits=[p.relative_to(ROOT).as_posix() for p in ROOT.rglob(pat) if "artifacts/" not in p.as_posix()]
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
