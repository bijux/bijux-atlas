#!/usr/bin/env python3
"""Convert benchmark history JSON into a plotting-friendly TSV."""

import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: benchmark_history_plot.py <history.json> <output.tsv>")
        return 2
    history_path = Path(sys.argv[1])
    out_path = Path(sys.argv[2])
    payload = json.loads(history_path.read_text())
    runs = payload.get("runs", [])
    lines = ["run_index\tlatency_p99_ms\tthroughput_rps\terror_rate_percent"]
    for run in runs:
        lines.append(
            f"{run.get('run_index', 0)}\t{run.get('latency_p99_ms', 0)}\t{run.get('throughput_rps', 0)}\t{run.get('error_rate_percent', 0)}"
        )
    out_path.write_text("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
