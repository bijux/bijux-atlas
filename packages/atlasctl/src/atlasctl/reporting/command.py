from __future__ import annotations

import argparse
import json
from ..reporting.make_area_report import main as make_area_report_main
from ..core.context import RunContext
from ..commands.policies.runtime.dir_entry_budgets import report_budgets
from .actions import (
    _cmd_artifact_gc,
    _cmd_artifact_index,
    _cmd_collect,
    _cmd_diff,
    _cmd_export,
    _cmd_junit,
    _cmd_last_fail,
    _cmd_print,
    _cmd_pr_summary,
    _cmd_scorecard,
    _cmd_summarize,
    _cmd_trend,
    _cmd_triage,
    _cmd_validate,
)


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
    if ns.report_cmd == "budgets":
        payload = report_budgets(ctx.repo_root, by_domain=bool(getattr(ns, "by_domain", False)))
        text = json.dumps(payload, sort_keys=True) if bool(getattr(ns, "json", False)) else json.dumps(payload, indent=2, sort_keys=True)
        if ns.out:
            out = ctx.repo_root / ns.out
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(text + "\n", encoding="utf-8")
        print(text)
        return 0 if payload.get("status") == "ok" else 1
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
    budgets = rep.add_parser("budgets", help="ranked directory budget report")
    budgets.add_argument("--by-domain", action="store_true", help="aggregate budget offenders by top-level domain")
    budgets.add_argument("--json", action="store_true", help="emit JSON output")
    budgets.add_argument("--out", help="write output to file path")

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
