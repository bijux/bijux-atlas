#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from xml.etree.ElementTree import Element, SubElement, tostring

ROOT = Path(__file__).resolve().parents[2]
MAKE_ROOT = ROOT / "ops" / "_generated" / "make"


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
    summary = {
        "total": len(lanes),
        "passed": sum(1 for v in lanes.values() if v.get("status") == "pass"),
        "failed": sum(1 for v in lanes.values() if v.get("status") == "fail"),
    }
    return {
        "schema_version": 1,
        "run_id": run_id,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "lanes": lanes,
        "summary": summary,
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
        "| lane | status | failure | log |",
        "|---|---|---|---|",
    ]
    for lane, report in sorted(payload["lanes"].items()):
        fail = (report.get("failure_summary") or "").replace("|", "/")
        lines.append(f"| {lane} | {report.get('status','unknown')} | {fail} | {report.get('log','-')} |")
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


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("command", choices=["merge", "print", "md", "junit"])
    p.add_argument("--run-id", required=True)
    args = p.parse_args()

    if args.command == "merge":
        return cmd_merge(args.run_id)
    if args.command == "print":
        return cmd_print(args.run_id)
    if args.command == "md":
        return cmd_md(args.run_id)
    return cmd_junit(args.run_id)


if __name__ == "__main__":
    raise SystemExit(main())
