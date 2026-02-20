from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--path", required=True)
    p.add_argument("--lane", required=True)
    p.add_argument("--run-id", required=False, default="local")
    p.add_argument("--status", required=True)
    p.add_argument("--start", required=True)
    p.add_argument("--end", required=True)
    p.add_argument("--duration-seconds", type=float, default=0.0)
    p.add_argument("--log", default="-")
    p.add_argument("--artifact", action="append", default=[])
    p.add_argument("--failure", default="")
    args = p.parse_args(argv)

    out = Path(args.path)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": 1,
        "report_version": 1,
        "lane": args.lane,
        "run_id": args.run_id,
        "status": args.status,
        "started_at": args.start,
        "ended_at": args.end,
        "duration_seconds": args.duration_seconds,
        "log": args.log,
        "artifact_paths": args.artifact,
        "failure_summary": args.failure,
    }
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

