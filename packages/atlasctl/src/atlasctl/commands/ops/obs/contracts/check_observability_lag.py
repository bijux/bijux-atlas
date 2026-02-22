#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import json
import subprocess
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
BUDGETS = ROOT / "configs/ops/obs/budgets.json"
LAST_PASS = ROOT / "artifacts/evidence/obs/last-pass.json"


def _current_branch() -> str:
    try:
        return subprocess.check_output(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=ROOT, text=True
        ).strip()
    except Exception:
        return ""


def main() -> int:
    budgets = json.loads(BUDGETS.read_text(encoding="utf-8"))
    max_days = int(budgets.get("lag", {}).get("max_days_since_pass_on_main", 7))

    branch = _current_branch()
    if branch not in {"main", "master"}:
        print(f"observability lag check skipped on branch `{branch or 'unknown'}`")
        return 0

    if not LAST_PASS.exists():
        print(
            f"observability lag check failed: missing {LAST_PASS.relative_to(ROOT)} on {branch}",
            file=sys.stderr,
        )
        return 1

    payload = json.loads(LAST_PASS.read_text(encoding="utf-8"))
    ts = payload.get("timestamp_utc")
    if not isinstance(ts, str) or not ts:
        print("observability lag check failed: last-pass timestamp missing", file=sys.stderr)
        return 1

    last_pass = dt.datetime.strptime(ts, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=dt.timezone.utc)
    now = dt.datetime.now(dt.timezone.utc)
    age_days = (now - last_pass).total_seconds() / 86400.0
    if age_days > max_days:
        print(
            f"observability lag check failed: last pass is {age_days:.1f} days old, budget is {max_days} days",
            file=sys.stderr,
        )
        return 1

    print(f"observability lag check passed (age={age_days:.1f}d <= {max_days}d)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
