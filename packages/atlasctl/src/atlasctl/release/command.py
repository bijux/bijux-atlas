from __future__ import annotations

import argparse
import json
import subprocess

from ..core.context import RunContext


def _run(ctx: RunContext, cmd: list[str]) -> tuple[bool, str]:
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False)
    ok = proc.returncode == 0
    output = (proc.stdout or proc.stderr).strip().splitlines()
    message = output[0] if output else ""
    return ok, message


def run_release_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.release_cmd != "checklist":
        return 2
    checks = [
        ("suite_refgrade", ["python3", "-m", "atlasctl.cli", "--quiet", "suite", "run", "refgrade", "--json"]),
        ("suite_refgrade_proof", ["python3", "-m", "atlasctl.cli", "--quiet", "suite", "run", "refgrade_proof", "--json"]),
        ("suite_release_0_1", ["python3", "-m", "atlasctl.cli", "--quiet", "suite", "run", "release_0_1", "--json"]),
        ("suite_inventory", ["python3", "-m", "atlasctl.cli", "--quiet", "suite", "check", "--json"]),
    ]
    rows: list[dict[str, object]] = []
    for check_id, cmd in checks:
        if ns.plan:
            rows.append({"id": check_id, "status": "planned", "command": cmd})
            continue
        ok, message = _run(ctx, cmd)
        rows.append({"id": check_id, "status": "pass" if ok else "fail", "command": cmd, "message": message})
    failed = [row for row in rows if row["status"] == "fail"]
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "release-checklist",
        "run_id": ctx.run_id,
        "status": "ok" if not failed else "error",
        "plan": bool(ns.plan),
        "checks": rows,
    }
    if ns.json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"release checklist: {payload['status']}")
        for row in rows:
            print(f"- {row['id']}: {row['status']}")
    return 0 if not failed else 1


def configure_release_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("release", help="release readiness commands")
    p_sub = p.add_subparsers(dest="release_cmd", required=True)
    checklist = p_sub.add_parser("checklist", help="print and run required release gates")
    checklist.add_argument("--plan", action="store_true", help="print release checks without executing them")
    checklist.add_argument("--json", action="store_true", help="emit JSON output")
