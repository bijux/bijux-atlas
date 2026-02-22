#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path


def main() -> int:
    import argparse

    p = argparse.ArgumentParser(description="Render k8s suite markdown summary")
    p.add_argument("--json", required=True)
    p.add_argument("--out", required=True)
    args = p.parse_args()

    payload = json.loads(Path(args.json).read_text(encoding="utf-8"))
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# K8s Suite Summary",
        "",
        f"- Run ID: `{payload.get('run_id', 'unknown')}`",
        f"- Suite: `{payload.get('suite_id', 'adhoc')}`",
        f"- Total: `{payload.get('total', 0)}`",
        f"- Passed: `{payload.get('passed', 0)}`",
        f"- Failed: `{payload.get('failed', 0)}`",
        f"- Skipped: `{payload.get('skipped', 0)}`",
        f"- Duration (s): `{payload.get('duration_seconds', 0)}`",
        "",
        "## Failures",
    ]
    failures = [r for r in payload.get("results", []) if r.get("status") == "failed"]
    if not failures:
        lines.append("- none")
    else:
        for f in failures:
            lines.append(
                f"- `{f.get('script')}` mode=`{f.get('observed_failure_mode')}` artifacts=`{f.get('artifacts_dir')}`"
            )
    lines.append("")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
