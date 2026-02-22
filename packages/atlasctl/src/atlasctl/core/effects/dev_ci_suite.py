from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from ...core.runtime.paths import write_text_file
from ..context import RunContext
from .exec import run as process_run
from .run_meta import write_run_meta


def run_suite_ci(
    ctx: RunContext,
    ns: argparse.Namespace,
    *,
    ci_out_dir,
    lane_for_label,
    lane_filters: dict[str, tuple[str, ...]],
) -> int:
    out_dir = ci_out_dir(ctx, getattr(ns, "out_dir", None))
    out_dir.mkdir(parents=True, exist_ok=True)
    junit_path = out_dir / "suite-ci.junit.xml"
    suite_cmd: list[str] = [
        sys.executable,
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
    ]
    lanes = list(getattr(ns, "lane", []) or [])
    if lanes:
        seen: set[str] = set()
        for lane in lanes:
            for pattern in lane_filters.get(lane, ()):
                if pattern not in seen:
                    seen.add(pattern)
                    suite_cmd.extend(["--only", pattern])
    fail_fast = bool(getattr(ns, "fail_fast", False))
    maxfail = max(0, int(getattr(ns, "maxfail", 0) or 0))
    suite_cmd.append("--fail-fast" if fail_fast else "--keep-going")
    if maxfail:
        suite_cmd.extend(["--maxfail", str(maxfail)])
    execution_mode = "fail-fast" if fail_fast else "keep-going"
    no_isolate = bool(getattr(ns, "no_isolate", False))
    isolate_mode = "debug-no-isolate" if no_isolate else "isolate"
    planned_cmd = suite_cmd if no_isolate else [
        sys.executable,
        "-m",
        "atlasctl.cli",
        "env",
        "isolate",
        "--tag",
        f"ci-{ctx.run_id}",
        *suite_cmd,
    ]
    if bool(getattr(ns, "explain", False)):
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "run_id": ctx.run_id,
            "action": "ci-run-explain",
            "lane_filter": lanes if lanes else ["all"],
            "mode": isolate_mode,
            "execution": execution_mode,
            "maxfail": maxfail,
            "artifacts": {
                "json": str(out_dir / "suite-ci.report.json"),
                "junit": str(junit_path),
                "summary": str(out_dir / "suite-ci.summary.txt"),
            },
            "planned_steps": [{"id": "ci.step.001", "command": " ".join(planned_cmd)}],
        }
        if ns.json or ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"ci run plan: mode={isolate_mode} execution={execution_mode} lanes={','.join(lanes) if lanes else 'all'}")
            print(f"- {' '.join(planned_cmd)}")
        return 0

    cmd = suite_cmd if no_isolate else [sys.executable, "-m", "atlasctl.cli", "env", "isolate", "--tag", f"ci-{ctx.run_id}", *suite_cmd]
    proc = process_run(cmd, cwd=ctx.repo_root, text=True, capture_output=True)
    if proc.stdout.strip():
        try:
            payload = json.loads(proc.stdout)
        except json.JSONDecodeError:
            payload = {
                "status": "error",
                "summary": {"passed": 0, "failed": 1, "skipped": 0},
                "errors": [
                    "runtime error: suite output was not valid JSON. "
                    "Next: rerun with `atlasctl dev ci run --verbose --no-isolate`."
                ],
            }
    else:
        payload = {"status": "error", "summary": {"passed": 0, "failed": 1, "skipped": 0}}

    suite_steps = [
        {
            "id": f"ci.step.{idx:03d}",
            "lane": lane_for_label(str(row.get("label", ""))),
            "label": str(row.get("label", "")),
            "status": str(row.get("status", "unknown")),
        }
        for idx, row in enumerate(payload.get("results", []), start=1)
    ]
    report_path = out_dir / "suite-ci.report.json"
    summary_path = out_dir / "suite-ci.summary.txt"
    write_text_file(report_path, json.dumps(payload, indent=2, sort_keys=True) + "\n")
    summary = payload.get("summary", {})
    summary_txt = (
        f"run_id={ctx.run_id}\n"
        f"status={payload.get('status','error')}\n"
        f"passed={summary.get('passed', 0)} failed={summary.get('failed', 0)} skipped={summary.get('skipped', 0)}\n"
        f"lanes={','.join(lanes) if lanes else 'all'}\n"
        f"junit={junit_path}\n"
        f"json={report_path}\n"
    )
    write_text_file(summary_path, summary_txt)
    meta_path = write_run_meta(ctx, out_dir, lane="ci")
    if ns.json or ctx.output_format == "json":
        print(
            json.dumps(
                {
                    "schema_version": 1,
                    "tool": "atlasctl",
                    "status": "ok" if proc.returncode == 0 else "error",
                    "run_id": ctx.run_id,
                    "lane_filter": lanes if lanes else ["all"],
                    "mode": isolate_mode,
                    "execution": execution_mode,
                    "maxfail": maxfail,
                    "suite_result": payload,
                    "suite_steps": suite_steps,
                    "next": (
                        "rerun with `atlasctl dev ci run --verbose --no-isolate` for step-level diagnostics"
                        if proc.returncode != 0
                        else ""
                    ),
                    "artifacts": {
                        "json": str(report_path),
                        "junit": str(junit_path),
                        "summary": str(summary_path),
                        "meta": str(meta_path),
                    },
                },
                sort_keys=True,
            )
        )
    elif proc.returncode == 0:
        print(f"ci run: pass (suite ci) run_id={ctx.run_id}")
    else:
        print(f"ci run: fail (suite ci) run_id={ctx.run_id} (next: rerun with --verbose --no-isolate)")
    return 0 if proc.returncode == 0 else 1
