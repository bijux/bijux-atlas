from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from ..core.context import RunContext


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.ci_cmd == "scripts":
        proc = subprocess.run(["make", "-s", "scripts-check"], cwd=ctx.repo_root, text=True, check=False)
        return proc.returncode
    if ns.ci_cmd == "run":
        out_dir = Path(ns.out_dir) if ns.out_dir else (ctx.repo_root / "artifacts" / "evidence" / "ci" / ctx.run_id)
        out_dir.mkdir(parents=True, exist_ok=True)
        junit_path = out_dir / "suite-ci.junit.xml"
        proc = subprocess.run(
            [
                "python3",
                "-m",
                "atlasctl.cli",
                "--quiet",
                "--format",
                "json",
                "--run-id",
                ctx.run_id,
                "suite",
                "run",
                "ci",
                "--json",
                "--junit",
                str(junit_path),
            ],
            cwd=ctx.repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
        payload = json.loads(proc.stdout) if proc.stdout.strip() else {"status": "error", "summary": {"passed": 0, "failed": 1, "skipped": 0}}
        (out_dir / "suite-ci.report.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        summary = payload.get("summary", {})
        summary_txt = (
            f"run_id={ctx.run_id}\n"
            f"status={payload.get('status','error')}\n"
            f"passed={summary.get('passed', 0)} failed={summary.get('failed', 0)} skipped={summary.get('skipped', 0)}\n"
            f"junit={junit_path}\n"
            f"json={out_dir / 'suite-ci.report.json'}\n"
        )
        (out_dir / "suite-ci.summary.txt").write_text(summary_txt, encoding="utf-8")
        if ns.json or ctx.output_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "atlasctl",
                        "status": "ok" if proc.returncode == 0 else "error",
                        "run_id": ctx.run_id,
                        "suite_result": payload,
                        "artifacts": {
                            "json": str(out_dir / "suite-ci.report.json"),
                            "junit": str(junit_path),
                            "summary": str(out_dir / "suite-ci.summary.txt"),
                        },
                    },
                    sort_keys=True,
                )
            )
        elif proc.returncode == 0:
            print(f"ci run: pass (suite ci) run_id={ctx.run_id}")
        else:
            print(f"ci run: fail (suite ci) run_id={ctx.run_id}")
        return proc.returncode
    return 2


def configure_ci_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ci", help="ci command group")
    ci_sub = p.add_subparsers(dest="ci_cmd", required=True)
    ci_sub.add_parser("scripts", help="run scripts ci checks")
    run = ci_sub.add_parser("run", help="run canonical CI suite locally")
    run.add_argument("--json", action="store_true", help="emit JSON output")
    run.add_argument("--out-dir", help="output directory for CI artifacts")
