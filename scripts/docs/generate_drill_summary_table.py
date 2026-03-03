#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Render a markdown table from ops drill summary JSON.")
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)

    if input_path.exists():
        payload = json.loads(input_path.read_text())
        rows = payload.get("drills", [])
    else:
        rows = []

    rows = sorted(rows, key=lambda row: row.get("name", ""))
    lines = [
        "# Drill Summary",
        "",
        "| Drill | Status | Report |",
        "| --- | --- | --- |",
    ]
    for row in rows:
        name = row.get("name", "")
        status = row.get("status", "")
        report = row.get("report_path", "")
        lines.append(f"| {name} | {status} | `{report}` |")
    if not rows:
        lines.append("| (none) | n/a | n/a |")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
