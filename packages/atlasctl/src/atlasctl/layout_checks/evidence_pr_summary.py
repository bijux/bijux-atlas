#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def main() -> int:
    run_id = (ROOT / "artifacts/evidence/latest-run-id.txt").read_text(encoding="utf-8").strip()
    unified = ROOT / "artifacts/evidence" / "make" / run_id / "unified.json"
    data = json.loads(unified.read_text(encoding="utf-8"))
    out = ROOT / "artifacts/evidence" / "make" / run_id / "pr-summary.md"
    lines = [
        f"## Ops Evidence Summary (`{run_id}`)",
        "",
        f"- total lanes: `{data['summary']['total']}`",
        f"- passed: `{data['summary']['passed']}`",
        f"- failed: `{data['summary']['failed']}`",
        "",
        "| lane | status |",
        "|---|---|",
    ]
    for lane, payload in sorted(data.get("lanes", {}).items()):
        lines.append(f"| {lane} | {payload.get('status', 'unknown')} |")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
