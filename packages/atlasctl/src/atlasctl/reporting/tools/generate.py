#!/usr/bin/env python3
# Purpose: render markdown summary from unified ops report JSON.
# Inputs: --unified path to ops/_generated.example/report.unified.json.
# Outputs: markdown summary file.
from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--unified", required=True, help="path to unified ops report JSON")
    parser.add_argument("--out", required=True, help="output markdown path")
    args = parser.parse_args()

    unified_path = Path(args.unified)
    payload = json.loads(unified_path.read_text(encoding="utf-8"))
    summary = payload.get("summary", {})
    lanes = payload.get("lanes", {})

    lines = [
        "# Unified Ops Report",
        "",
        f"- run_id: `{payload.get('run_id', 'unknown')}`",
        f"- generated_at: `{payload.get('generated_at', 'unknown')}`",
        f"- total lanes: `{summary.get('total', 0)}`",
        f"- passed: `{summary.get('passed', 0)}`",
        f"- failed: `{summary.get('failed', 0)}`",
        "",
        "## Lanes",
        "",
    ]
    for lane, lane_report in sorted(lanes.items()):
        lines.append(
            f"- `{lane}`: status=`{lane_report.get('status','unknown')}` duration=`{lane_report.get('duration_seconds',0)}` log=`{lane_report.get('log','')}`"
        )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
