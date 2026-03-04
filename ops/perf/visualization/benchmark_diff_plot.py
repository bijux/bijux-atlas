#!/usr/bin/env python3
"""Render a simple benchmark delta chart from a diff JSON payload."""

import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: benchmark_diff_plot.py <diff.json> <output.tsv>")
        return 2
    diff_path = Path(sys.argv[1])
    out_path = Path(sys.argv[2])
    payload = json.loads(diff_path.read_text())
    row = payload.get("rows", [{}])[0]
    changes = row.get("changes", {})
    lines = ["metric\tfrom\tto\tregressed"]
    for metric, data in changes.items():
        lines.append(
            f"{metric}\t{data.get('from', 0)}\t{data.get('to', 0)}\t{data.get('regressed', False)}"
        )
    out_path.write_text("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
