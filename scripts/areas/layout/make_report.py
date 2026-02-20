#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from xml.etree.ElementTree import Element, SubElement, tostring

ROOT = Path(__file__).resolve().parents[3]
MAKE_ROOT = ROOT / "artifacts" / "evidence" / "make"


def discover_lane_reports(run_id: str) -> dict[str, dict]:
    reports: dict[str, dict] = {}
    if not MAKE_ROOT.exists():
        return reports

    pattern = f"*/{run_id}/report.json"
    for report_path in MAKE_ROOT.glob(pattern):
        lane = report_path.parent.parent.name
        reports[lane] = json.loads(report_path.read_text(encoding="utf-8"))

    nested_pattern = f"*/*/{run_id}/report.json"
    for report_path in MAKE_ROOT.glob(nested_pattern):
        rel = report_path.relative_to(MAKE_ROOT)
        lane = "/".join(rel.parts[:-2])
        reports[lane] = json.loads(report_path.read_text(encoding="utf-8"))

    return reports


def build_unified(run_id: str) -> dict:
    lanes = discover_lane_reports(run_id)
    near = []
    failed = []
    checked = 0
    for lane, report in lanes.items():
        budget = report.get("budget_status")
        if isinstance(budget, dict) and budget.get("checked"):
            checked += 1
            if budget.get("near_failing"):
                near.append(lane)
            if budget.get("status") == "fail":
                failed.append(lane)
    summary = {
        "total": len(lanes),
        "passed": sum(1 for v in lanes.values() if v.get("status") == "pass"),
        "failed": sum(1 for v in lanes.values() if v.get("status") == "fail"),
    }
    budget_status = {
        "checked": checked,
        "failed": len(failed),
        "near_failing": sorted(near),
        "failed_lanes": sorted(failed),
    }
    perf_summary = {"suite_count": 0, "p95_max_ms": 0.0, "p99_max_ms": 0.0}
    perf_raw = ROOT / "artifacts" / "evidence" / "perf" / run_id / "raw"
    if perf_raw.exists():
        p95s = []
        p99s = []
        for summary in sorted(perf_raw.glob("*.summary.json")):
            data = json.loads(summary.read_text(encoding="utf-8"))
            vals = data.get("metrics", {}).get("http_req_duration", {}).get("values", {})
            p95s.append(float(vals.get("p(95)", 0.0)))
            p99s.append(float(vals.get("p(99)", 0.0)))
        if p95s:
            perf_summary = {
                "suite_count": len(p95s),
                "p95_max_ms": max(p95s),
                "p99_max_ms": max(p99s) if p99s else 0.0,
            }
    graceful_degradation = {"status": "fail", "score_percent": 0.0, "total_considered": 0, "failed": 0}
    gd_path = ROOT / "artifacts" / "evidence" / "k8s" / run_id / "graceful-degradation-score.json"
    if gd_path.exists():
        graceful_degradation = json.loads(gd_path.read_text(encoding="utf-8"))
    k8s_conformance = {"status": "fail", "failed_sections": []}
    kc_path = ROOT / "artifacts" / "evidence" / "k8s" / run_id / "k8s-conformance-report.json"
    if kc_path.exists():
        k8s_conformance = json.loads(kc_path.read_text(encoding="utf-8"))
    return {
        "schema_version": 1,
        "run_id": run_id,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "lanes": lanes,
        "summary": summary,
        "budget_status": budget_status,
        "perf_summary": perf_summary,
        "graceful_degradation": graceful_degradation,
        "k8s_conformance": k8s_conformance,
    }


def out_dir(run_id: str) -> Path:
    return MAKE_ROOT / run_id


def write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def cmd_merge(run_id: str) -> int:
    payload = build_unified(run_id)
    out = out_dir(run_id) / "unified.json"
    write_json(out, payload)
    print(out.relative_to(ROOT))
    return 0


def cmd_print(run_id: str) -> int:
    payload = build_unified(run_id)
    print(f"make report summary: run_id={run_id}")
    print(f"total={payload['summary']['total']} passed={payload['summary']['passed']} failed={payload['summary']['failed']}")
    for lane, report in sorted(payload["lanes"].items()):
        print(f"- {lane}: {report.get('status','unknown')} ({report.get('log','-')})")
        if report.get("status") != "pass" and report.get("repro_command"):
            print(f"  repro: {report.get('repro_command')}")
    b = payload.get("budget_status", {})
    if b.get("checked", 0):
        print(
            f"budget-status: checked={b.get('checked',0)} failed={b.get('failed',0)} near_failing={len(b.get('near_failing',[]))}"
        )
        if b.get("near_failing"):
            print("near-failing lanes: " + ", ".join(sorted(b["near_failing"])))
    return 0


def cmd_md(run_id: str) -> int:
    payload = build_unified(run_id)
    out = out_dir(run_id) / "summary.md"
    lines = [
        "# Make Lane Summary",
        "",
        f"- run_id: `{run_id}`",
        f"- total: `{payload['summary']['total']}`",
        f"- passed: `{payload['summary']['passed']}`",
        f"- failed: `{payload['summary']['failed']}`",
        "",
        "| lane | status | failure | repro | log |",
        "|---|---|---|---|---|",
    ]
    for lane, report in sorted(payload["lanes"].items()):
        fail = (report.get("failure_summary") or "").replace("|", "/")
        repro = (report.get("repro_command") or "").replace("|", "/")
        lines.append(f"| {lane} | {report.get('status','unknown')} | {fail} | `{repro}` | {report.get('log','-')} |")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out.relative_to(ROOT))
    return 0


def cmd_junit(run_id: str) -> int:
    payload = build_unified(run_id)
    suite = Element("testsuite", name="make-lanes", tests=str(payload["summary"]["total"]), failures=str(payload["summary"]["failed"]))
    for lane, report in sorted(payload["lanes"].items()):
        case = SubElement(suite, "testcase", classname="make.lanes", name=lane)
        if report.get("status") != "pass":
            failure = SubElement(case, "failure", message="lane failed")
            failure.text = report.get("failure_summary") or report.get("log") or "lane failed"
    out = out_dir(run_id) / "junit.xml"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(tostring(suite, encoding="unicode"), encoding="utf-8")
    print(out.relative_to(ROOT))
    return 0


def cmd_last_fail(run_id: str) -> int:
    payload = build_unified(run_id)
    failed = [(lane, report) for lane, report in sorted(payload["lanes"].items()) if report.get("status") != "pass"]
    if not failed:
        print(f"no failed lanes for run_id={run_id}")
        return 0
    lane, report = failed[-1]
    log_path = ROOT / str(report.get("log", ""))
    print(f"last-failed lane: {lane}")
    print(f"log: {log_path.relative_to(ROOT) if log_path.exists() else report.get('log','-')}")
    if report.get("repro_command"):
        print(f"repro: {report.get('repro_command')}")
    if log_path.exists():
        print("\n--- last 20 log lines ---")
        tail = log_path.read_text(encoding="utf-8", errors="replace").splitlines()[-20:]
        for line in tail:
            print(line)
    return 0


def cmd_triage(run_id: str) -> int:
    payload = build_unified(run_id)
    failed = [(lane, report) for lane, report in sorted(payload["lanes"].items()) if report.get("status") != "pass"]
    print(f"triage run_id={run_id} failed={len(failed)}")
    print(f"evidence: {(out_dir(run_id) / 'unified.json').relative_to(ROOT)}")
    for lane, report in failed:
        print(f"\n## {lane}")
        print(f"log: {report.get('log','-')}")
        if report.get("repro_command"):
            print(f"repro: {report.get('repro_command')}")
        log_path = ROOT / str(report.get("log", ""))
        if log_path.exists():
            print("--- last 20 lines ---")
            for line in log_path.read_text(encoding="utf-8", errors="replace").splitlines()[-20:]:
                print(line)
    return 1 if failed else 0


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("command", choices=["merge", "print", "md", "junit", "last-fail", "triage"])
    p.add_argument("--run-id", required=True)
    args = p.parse_args()

    if args.command == "merge":
        return cmd_merge(args.run_id)
    if args.command == "print":
        return cmd_print(args.run_id)
    if args.command == "md":
        return cmd_md(args.run_id)
    if args.command == "junit":
        return cmd_junit(args.run_id)
    if args.command == "last-fail":
        return cmd_last_fail(args.run_id)
    return cmd_triage(args.run_id)


if __name__ == "__main__":
    raise SystemExit(main())
