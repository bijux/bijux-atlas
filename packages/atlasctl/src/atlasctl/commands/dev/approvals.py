from __future__ import annotations

import argparse
import json
from pathlib import Path

from ...core.context import RunContext


APPROVALS_PATH = Path("configs/policy/check_speed_approvals.json")


def run_approvals_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.approvals_cmd != "add":
        return 2
    check_id = str(getattr(ns, "check_speed", "") or "").strip()
    max_ms = int(getattr(ns, "max_ms", 0) or 0)
    if not check_id or max_ms <= 0:
        print("usage: atlasctl approvals add --check-speed <id> --max-ms <n>")
        return 2
    path = ctx.repo_root / APPROVALS_PATH
    payload: dict[str, object] = {"schema_version": 1, "checks": {}}
    if path.exists():
        payload = json.loads(path.read_text(encoding="utf-8"))
    checks = payload.setdefault("checks", {})
    assert isinstance(checks, dict)
    checks[check_id] = max_ms
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "path": str(APPROVALS_PATH), "check_id": check_id, "max_ms": max_ms}, sort_keys=True))
    return 0


def configure_approvals_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("approvals", help="manage policy approval records")
    s = p.add_subparsers(dest="approvals_cmd", required=True)
    add = s.add_parser("add", help="add a check-speed approval")
    add.add_argument("--check-speed", required=True, help="check id")
    add.add_argument("--max-ms", required=True, type=int, help="approved max duration in ms")

