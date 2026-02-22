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


def _read(run_id: str) -> dict[str, float]:
    data_out: dict[str, float] = {}
    root = ROOT / "artifacts/evidence/perf" / run_id / "raw"
    for file in sorted(root.glob("*.summary.json")):
        data = json.loads(file.read_text(encoding="utf-8"))
        data_out[file.stem.replace(".summary", "")] = float(
            data.get("metrics", {}).get("http_req_duration", {}).get("values", {}).get("p(95)", 0.0)
        )
    return data_out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--from-run", required=True)
    parser.add_argument("--to-run", required=True)
    args = parser.parse_args()

    frm = _read(args.from_run)
    to = _read(args.to_run)
    suites = sorted(set(frm) | set(to))
    print(f"perf compare: from={args.from_run} to={args.to_run}")
    for suite in suites:
        a = frm.get(suite, 0.0)
        b = to.get(suite, 0.0)
        print(f"- {suite}: {a:.2f} -> {b:.2f} ({(b-a):+.2f} ms)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
