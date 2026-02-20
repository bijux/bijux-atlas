from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tarfile
from datetime import datetime, timedelta, timezone
from pathlib import Path
from xml.etree.ElementTree import Element, SubElement, tostring

from ..core.context import RunContext
from ..reporting.make_area_report import main as make_area_report_main


def _make_root(ctx: RunContext) -> Path:
    return ctx.evidence_root / "make"


def _discover_lane_reports(ctx: RunContext, run_id: str) -> dict[str, dict]:
    reports: dict[str, dict] = {}
    make_root = _make_root(ctx)
    if not make_root.exists():
        return reports

    for report_path in make_root.glob(f"*/{run_id}/report.json"):
        lane = report_path.parent.parent.name
        reports[lane] = json.loads(report_path.read_text(encoding="utf-8"))

    for report_path in make_root.glob(f"*/*/{run_id}/report.json"):
        rel = report_path.relative_to(make_root)
        lane = "/".join(rel.parts[:-2])
        reports[lane] = json.loads(report_path.read_text(encoding="utf-8"))
    return reports


def build_unified(ctx: RunContext, run_id: str) -> dict[str, object]:
    lanes = _discover_lane_reports(ctx, run_id)
    near: list[str] = []
    failed_budget: list[str] = []
    checked = 0
    for lane, report in lanes.items():
        budget = report.get("budget_status")
        if isinstance(budget, dict) and budget.get("checked"):
            checked += 1
            if budget.get("near_failing"):
                near.append(lane)
            if budget.get("status") == "fail":
                failed_budget.append(lane)

    summary = {
        "total": len(lanes),
        "passed": sum(1 for v in lanes.values() if v.get("status") == "pass"),
        "failed": sum(1 for v in lanes.values() if v.get("status") == "fail"),
    }
    budget_status = {
        "checked": checked,
        "failed": len(failed_budget),
        "near_failing": sorted(near),
        "failed_lanes": sorted(failed_budget),
    }

    perf_summary: dict[str, object] = {"suite_count": 0, "p95_max_ms": 0.0, "p99_max_ms": 0.0}
    perf_raw = ctx.evidence_root / "perf" / run_id / "raw"
    if perf_raw.exists():
        p95s: list[float] = []
        p99s: list[float] = []
        for summary_file in sorted(perf_raw.glob("*.summary.json")):
            data = json.loads(summary_file.read_text(encoding="utf-8"))
            vals = data.get("metrics", {}).get("http_req_duration", {}).get("values", {})
            p95s.append(float(vals.get("p(95)", 0.0)))
            p99s.append(float(vals.get("p(99)", 0.0)))
        if p95s:
            perf_summary = {
                "suite_count": len(p95s),
                "p95_max_ms": max(p95s),
                "p99_max_ms": max(p99s) if p99s else 0.0,
            }

    graceful_degradation: dict[str, object] = {
        "status": "fail",
        "score_percent": 0.0,
        "total_considered": 0,
        "failed": 0,
    }
    gd_path = ctx.evidence_root / "k8s" / run_id / "graceful-degradation-score.json"
    if gd_path.exists():
        graceful_degradation = json.loads(gd_path.read_text(encoding="utf-8"))

    k8s_conformance: dict[str, object] = {"status": "fail", "failed_sections": []}
    kc_path = ctx.evidence_root / "k8s" / run_id / "k8s-conformance-report.json"
    if kc_path.exists():
        k8s_conformance = json.loads(kc_path.read_text(encoding="utf-8"))

    return {
        "schema_version": 1,
        "report_version": 1,
        "run_id": run_id,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "lanes": lanes,
        "summary": summary,
        "budget_status": budget_status,
        "perf_summary": perf_summary,
        "graceful_degradation": graceful_degradation,
        "k8s_conformance": k8s_conformance,
    }


def _run_dir(ctx: RunContext, run_id: str) -> Path:
    return _make_root(ctx) / run_id


def _write_json(path: Path, data: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _cmd_collect(ctx: RunContext, run_id: str, out: str | None) -> int:
    payload = build_unified(ctx, run_id)
    out_path = Path(out) if out else (_run_dir(ctx, run_id) / "unified.json")
    _write_json(out_path, payload)
    print(out_path)
    return 0


def _cmd_validate(ctx: RunContext, run_id: str, file_path: str | None) -> int:
    import jsonschema

    unified = Path(file_path) if file_path else (_run_dir(ctx, run_id) / "unified.json")
    payload = json.loads(unified.read_text(encoding="utf-8"))
    schema_path = ctx.repo_root / "ops/_schemas/report/unified.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    print("ok")
    return 0


def _cmd_summarize(ctx: RunContext, run_id: str, out: str | None) -> int:
    payload = build_unified(ctx, run_id)
    out_path = Path(out) if out else (_run_dir(ctx, run_id) / "summary.md")
    lines = [
        "# Unified Report Summary",
        "",
        f"- run_id: `{run_id}`",
        f"- total: `{payload['summary']['total']}`",
        f"- passed: `{payload['summary']['passed']}`",
        f"- failed: `{payload['summary']['failed']}`",
        "",
        "| lane | status | failure | repro | log |",
        "|---|---|---|---|---|",
    ]
    lanes = payload.get("lanes", {})
    if isinstance(lanes, dict):
        for lane, report in sorted(lanes.items()):
            if not isinstance(report, dict):
                continue
            fail = str(report.get("failure_summary", "")).replace("|", "/")
            repro = str(report.get("repro_command", "")).replace("|", "/")
            status = report.get("status", "unknown")
            log = report.get("log", "-")
            lines.append(f"| {lane} | {status} | {fail} | `{repro}` | {log} |")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out_path)
    return 0


def _cmd_pr_summary(ctx: RunContext, run_id: str, out: str | None) -> int:
    payload = build_unified(ctx, run_id)
    out_path = Path(out) if out else (_run_dir(ctx, run_id) / "pr-summary.md")
    lines = [
        f"### bijux-atlas run `{run_id}`",
        "",
        f"- Total lanes: {payload['summary']['total']}",
        f"- Passed: {payload['summary']['passed']}",
        f"- Failed: {payload['summary']['failed']}",
        "",
    ]
    lanes = payload.get("lanes", {})
    if isinstance(lanes, dict):
        for lane, report in sorted(lanes.items()):
            if not isinstance(report, dict):
                continue
            status = report.get("status", "unknown")
            emoji = "✅" if status == "pass" else "❌"
            lines.append(f"- {emoji} `{lane}`: {status}")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out_path)
    return 0


def _cmd_print(ctx: RunContext, run_id: str) -> int:
    payload = build_unified(ctx, run_id)
    print(f"make report summary: run_id={run_id}")
    summary = payload["summary"]
    print(f"total={summary['total']} passed={summary['passed']} failed={summary['failed']}")
    lanes = payload.get("lanes", {})
    if isinstance(lanes, dict):
        for lane, report in sorted(lanes.items()):
            if not isinstance(report, dict):
                continue
            print(f"- {lane}: {report.get('status', 'unknown')} ({report.get('log', '-')})")
            if report.get("status") != "pass" and report.get("repro_command"):
                print(f"  repro: {report.get('repro_command')}")
    return 0


def _cmd_scorecard(ctx: RunContext, run_id: str, out: str | None) -> int:
    unified = _run_dir(ctx, run_id) / "unified.json"
    if not unified.exists():
        _cmd_collect(ctx, run_id, str(unified))
    out_path = Path(out) if out else (ctx.repo_root / "ops/_generated_committed/scorecard.json")
    cmd = [
        "python3",
        "./ops/report/make_confidence_scorecard.py",
        "--unified",
        str(unified),
        "--out",
        str(out_path),
        "--print-summary",
    ]
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False)
    if proc.stdout:
        print(proc.stdout.strip())
    if proc.returncode != 0 and proc.stderr:
        print(proc.stderr.strip())
    return proc.returncode


def _cmd_junit(ctx: RunContext, run_id: str, out: str | None) -> int:
    payload = build_unified(ctx, run_id)
    summary = payload["summary"]
    suite = Element("testsuite", name="make-lanes", tests=str(summary["total"]), failures=str(summary["failed"]))
    lanes = payload.get("lanes", {})
    if isinstance(lanes, dict):
        for lane, report in sorted(lanes.items()):
            case = SubElement(suite, "testcase", classname="make.lanes", name=lane)
            if isinstance(report, dict) and report.get("status") != "pass":
                failure = SubElement(case, "failure", message="lane failed")
                failure.text = str(report.get("failure_summary") or report.get("log") or "lane failed")
    out_path = Path(out) if out else (_run_dir(ctx, run_id) / "junit.xml")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(tostring(suite, encoding="unicode"), encoding="utf-8")
    print(out_path)
    return 0


def _cmd_last_fail(ctx: RunContext, run_id: str) -> int:
    payload = build_unified(ctx, run_id)
    lanes = payload.get("lanes", {})
    failed: list[tuple[str, dict[str, object]]] = []
    if isinstance(lanes, dict):
        for lane, report in sorted(lanes.items()):
            if isinstance(report, dict) and report.get("status") != "pass":
                failed.append((lane, report))
    if not failed:
        print(f"no failed lanes for run_id={run_id}")
        return 0
    lane, report = failed[-1]
    log_raw = str(report.get("log", ""))
    log_path = ctx.repo_root / log_raw if log_raw else Path("")
    print(f"last-failed lane: {lane}")
    print(f"log: {log_raw or '-'}")
    if report.get("repro_command"):
        print(f"repro: {report.get('repro_command')}")
    if log_raw and log_path.exists():
        print("\n--- last 20 log lines ---")
        for line in log_path.read_text(encoding="utf-8", errors="replace").splitlines()[-20:]:
            print(line)
    return 0


def _cmd_triage(ctx: RunContext, run_id: str) -> int:
    payload = build_unified(ctx, run_id)
    lanes = payload.get("lanes", {})
    failed: list[tuple[str, dict[str, object]]] = []
    if isinstance(lanes, dict):
        for lane, report in sorted(lanes.items()):
            if isinstance(report, dict) and report.get("status") != "pass":
                failed.append((lane, report))
    print(f"triage run_id={run_id} failed={len(failed)}")
    print(f"evidence: {_run_dir(ctx, run_id) / 'unified.json'}")
    for lane, report in failed:
        print(f"\n## {lane}")
        print(f"log: {report.get('log', '-')}")
        if report.get("repro_command"):
            print(f"repro: {report.get('repro_command')}")
    return 1 if failed else 0


def _load_unified(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _cmd_diff(ctx: RunContext, from_run: str, to_run: str) -> int:
    old = _load_unified(_run_dir(ctx, from_run) / "unified.json")
    new = _load_unified(_run_dir(ctx, to_run) / "unified.json")
    old_lanes = old.get("lanes", {})
    new_lanes = new.get("lanes", {})
    changed: list[str] = []
    if isinstance(new_lanes, dict):
        for lane, rep in sorted(new_lanes.items()):
            old_status = old_lanes.get(lane, {}).get("status") if isinstance(old_lanes, dict) else None
            new_status = rep.get("status") if isinstance(rep, dict) else None
            if old_status != new_status:
                changed.append(f"{lane}: {old_status} -> {new_status}")
    print(f"diff: {from_run} -> {to_run}")
    if not changed:
        print("no lane status changes")
        return 0
    for row in changed:
        print(f"- {row}")
    return 0


def _cmd_trend(ctx: RunContext, limit: int) -> int:
    make_root = _make_root(ctx)
    if not make_root.exists():
        print("no runs found")
        return 0
    rows: list[tuple[str, int, int]] = []
    for run_dir in sorted([p for p in make_root.iterdir() if p.is_dir()], reverse=True):
        unified = run_dir / "unified.json"
        if not unified.exists():
            continue
        payload = _load_unified(unified)
        summary = payload.get("summary", {})
        rows.append(
            (
                run_dir.name,
                int(summary.get("passed", 0)) if isinstance(summary, dict) else 0,
                int(summary.get("failed", 0)) if isinstance(summary, dict) else 0,
            )
        )
        if len(rows) >= limit:
            break
    for run_id, passed, failed in rows:
        print(f"{run_id}: passed={passed} failed={failed}")
    return 0


def _cmd_export(ctx: RunContext, run_id: str, out: str | None) -> int:
    run_dir = _run_dir(ctx, run_id)
    if not run_dir.exists():
        print(f"missing run dir: {run_dir}")
        return 1
    out_path = Path(out) if out else (run_dir / "evidence.tar.gz")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(out_path, "w:gz") as tar:
        tar.add(run_dir, arcname=run_dir.name)
    print(out_path)
    return 0


def _cmd_artifact_index(ctx: RunContext, limit: int, out: str | None) -> int:
    root = (ctx.repo_root / "artifacts/bijux-atlas-scripts/run").resolve()
    rows: list[dict[str, object]] = []
    if root.exists():
        candidates = sorted(
            [p for p in root.iterdir() if p.is_dir()],
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )
        for run in candidates[:limit]:
            rows.append(
                {
                    "run_id": run.name,
                    "path": str(run),
                    "reports": sorted(str(p.relative_to(ctx.repo_root)) for p in run.glob("reports/*.json")),
                    "logs": sorted(str(p.relative_to(ctx.repo_root)) for p in run.glob("logs/*.log")),
                }
            )
    payload = {"schema_version": 1, "tool": "bijux-atlas", "artifact_runs": rows}
    if out:
        out_path = Path(out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(out_path)
    else:
        print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


def _cmd_artifact_gc(ctx: RunContext, older_than_days: int | None) -> int:
    cfg = ctx.repo_root / "configs/ops/scripts-artifact-retention.json"
    payload = {"scripts_retention_days": 14}
    if cfg.exists():
        payload = json.loads(cfg.read_text(encoding="utf-8"))
    days = int(payload.get("scripts_retention_days", 14)) if older_than_days is None else int(older_than_days)
    root = (ctx.repo_root / "artifacts/bijux-atlas-scripts/run").resolve()
    removed: list[str] = []
    if root.exists():
        cutoff = datetime.now(timezone.utc) - timedelta(days=days)
        for run in sorted([p for p in root.iterdir() if p.is_dir()]):
            modified = datetime.fromtimestamp(run.stat().st_mtime, tz=timezone.utc)
            if modified < cutoff:
                shutil.rmtree(run, ignore_errors=True)
                removed.append(str(run))
    print(
        json.dumps(
            {
                "schema_version": 1,
                "tool": "bijux-atlas",
                "removed": sorted(removed),
                "retention_days": days,
            },
            sort_keys=True,
        )
    )
    return 0


def run_report_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.report_cmd == "collect":
        return _cmd_collect(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "unified":
        return _cmd_collect(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "validate":
        return _cmd_validate(ctx, ns.run_id_override or ctx.run_id, ns.file)
    if ns.report_cmd == "summarize":
        return _cmd_summarize(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "scorecard":
        return _cmd_scorecard(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "print":
        return _cmd_print(ctx, ns.run_id_override or ctx.run_id)
    if ns.report_cmd == "junit":
        return _cmd_junit(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "last-fail":
        return _cmd_last_fail(ctx, ns.run_id_override or ctx.run_id)
    if ns.report_cmd == "triage":
        return _cmd_triage(ctx, ns.run_id_override or ctx.run_id)
    if ns.report_cmd == "diff":
        return _cmd_diff(ctx, ns.from_run, ns.to_run)
    if ns.report_cmd == "trend":
        return _cmd_trend(ctx, ns.limit)
    if ns.report_cmd == "export":
        return _cmd_export(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "bundle":
        return _cmd_export(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "pr-summary":
        return _cmd_pr_summary(ctx, ns.run_id_override or ctx.run_id, ns.out)
    if ns.report_cmd == "artifact-index":
        return _cmd_artifact_index(ctx, ns.limit, ns.out)
    if ns.report_cmd == "artifact-gc":
        return _cmd_artifact_gc(ctx, ns.older_than_days)
    if ns.report_cmd == "make-area-write":
        argv = [
            "--path",
            ns.path,
            "--lane",
            ns.lane,
            "--run-id",
            ns.run_id,
            "--status",
            ns.status,
            "--start",
            ns.start,
            "--end",
            ns.end,
            "--duration-seconds",
            str(ns.duration_seconds),
            "--log",
            ns.log,
            "--failure",
            ns.failure,
        ]
        for artifact in ns.artifact:
            argv.extend(["--artifact", artifact])
        return make_area_report_main(argv)
    return 2


def configure_report_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("report", help="unified report and scorecard commands")
    rep = p.add_subparsers(dest="report_cmd", required=True)

    c = rep.add_parser("collect", help="collect lane reports into unified report JSON")
    c.add_argument("--run-id", dest="run_id_override")
    c.add_argument("--out")

    u = rep.add_parser("unified", help="alias for collect unified report JSON")
    u.add_argument("--run-id", dest="run_id_override")
    u.add_argument("--out")

    v = rep.add_parser("validate", help="validate unified report against schema")
    v.add_argument("--run-id", dest="run_id_override")
    v.add_argument("--file")

    s = rep.add_parser("summarize", help="render markdown summary for a run")
    s.add_argument("--run-id", dest="run_id_override")
    s.add_argument("--out")

    sc = rep.add_parser("scorecard", help="compute scorecard from unified report")
    sc.add_argument("--run-id", dest="run_id_override")
    sc.add_argument("--out")

    p1 = rep.add_parser("print", help="print one-screen summary")
    p1.add_argument("--run-id", dest="run_id_override")

    ju = rep.add_parser("junit", help="emit junit xml for run")
    ju.add_argument("--run-id", dest="run_id_override")
    ju.add_argument("--out")

    lf = rep.add_parser("last-fail", help="show last failed lane and tail")
    lf.add_argument("--run-id", dest="run_id_override")

    tr = rep.add_parser("triage", help="show failing lanes and repro info")
    tr.add_argument("--run-id", dest="run_id_override")

    d = rep.add_parser("diff", help="compare two unified reports by run id")
    d.add_argument("--from-run", required=True)
    d.add_argument("--to-run", required=True)

    t = rep.add_parser("trend", help="print recent pass/fail trend")
    t.add_argument("--limit", type=int, default=10)

    ex = rep.add_parser("export", help="export run evidence bundle")
    ex.add_argument("--run-id", dest="run_id_override")
    ex.add_argument("--out")

    bundle = rep.add_parser("bundle", help="alias for export run evidence bundle")
    bundle.add_argument("--run-id", dest="run_id_override")
    bundle.add_argument("--out")

    ps = rep.add_parser("pr-summary", help="write short PR-friendly summary markdown")
    ps.add_argument("--run-id", dest="run_id_override")
    ps.add_argument("--out")

    ai = rep.add_parser("artifact-index", help="list recent scripts artifact runs")
    ai.add_argument("--limit", type=int, default=10)
    ai.add_argument("--out")

    gc = rep.add_parser("artifact-gc", help="garbage collect scripts artifacts by retention")
    gc.add_argument("--older-than-days", type=int)

    mar = rep.add_parser("make-area-write", help="write lane make-area report JSON")
    mar.add_argument("--path", required=True)
    mar.add_argument("--lane", required=True)
    mar.add_argument("--run-id", required=True)
    mar.add_argument("--status", required=True)
    mar.add_argument("--start", required=True)
    mar.add_argument("--end", required=True)
    mar.add_argument("--duration-seconds", type=float, default=0.0)
    mar.add_argument("--log", default="-")
    mar.add_argument("--artifact", action="append", default=[])
    mar.add_argument("--failure", default="")
