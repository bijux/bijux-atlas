#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--path", required=True)
    p.add_argument("--lane", required=True)
    p.add_argument("--status", required=True)
    p.add_argument("--start", required=True)
    p.add_argument("--end", required=True)
    p.add_argument("--artifact", action="append", default=[])
    p.add_argument("--failure", default="")
    args = p.parse_args()

    out = Path(args.path)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "lane": args.lane,
        "status": args.status,
        "started_at": args.start,
        "ended_at": args.end,
        "artifact_paths": args.artifact,
        "failure_summary": args.failure,
    }
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
