#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _read(run_id: str) -> dict[str, float]:
    d: dict[str, float] = {}
    root = ROOT / "ops/_evidence/perf" / run_id / "raw"
    for f in sorted(root.glob("*.summary.json")):
        data = json.loads(f.read_text(encoding="utf-8"))
        d[f.stem.replace(".summary", "")] = float(
            data.get("metrics", {}).get("http_req_duration", {}).get("values", {}).get("p(95)", 0.0)
        )
    return d


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--from-run", required=True)
    p.add_argument("--to-run", required=True)
    args = p.parse_args()

    frm = _read(args.from_run)
    to = _read(args.to_run)
    suites = sorted(set(frm) | set(to))
    print(f"perf compare: from={args.from_run} to={args.to_run}")
    for s in suites:
        a = frm.get(s, 0.0)
        b = to.get(s, 0.0)
        print(f"- {s}: {a:.2f} -> {b:.2f} ({(b-a):+.2f} ms)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
