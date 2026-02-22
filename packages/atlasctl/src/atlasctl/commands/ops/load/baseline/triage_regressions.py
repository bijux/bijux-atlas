#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def _measured(results: Path) -> dict[str, float]:
    out: dict[str, float] = {}
    for file in sorted(results.glob("*.summary.json")):
        data = json.loads(file.read_text(encoding="utf-8"))
        out[file.stem.replace(".summary", "")] = float(
            data.get("metrics", {}).get("http_req_duration", {}).get("values", {}).get("p(95)", 0.0)
        )
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", default="local")
    parser.add_argument("--results", default="artifacts/perf/results")
    args = parser.parse_args()

    baseline = json.loads((ROOT / "configs/ops/perf/baselines" / f"{args.profile}.json").read_text(encoding="utf-8"))
    measured = _measured((ROOT / args.results).resolve())
    rows = []
    for row in baseline.get("rows", []):
        suite = str(row.get("suite", ""))
        if suite not in measured:
            continue
        delta = measured[suite] - float(row.get("p95_ms", 0))
        rows.append((delta, suite, float(row.get("p95_ms", 0)), measured[suite]))
    rows.sort(reverse=True)
    print("top regressions by endpoint/suite (p95 ms):")
    for delta, suite, base, now in rows[:10]:
        print(f"- {suite}: baseline={base:.2f} now={now:.2f} delta={delta:+.2f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
