from __future__ import annotations

import argparse
import json
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ....core.context import RunContext
from ....core.fs import ensure_evidence_path
from ....core.process import run_command
from ....core.runtime.paths import write_text_file


@dataclass(frozen=True)
class Lane:
    lane_id: str
    description: str
    make_target: str


def _load_lanes(repo_root: Path) -> tuple[dict[str, Lane], dict[str, list[str]]]:
    cfg = json.loads((repo_root / "configs/gates/lanes.json").read_text(encoding="utf-8"))
    lanes: dict[str, Lane] = {}
    for raw in cfg.get("lanes", []):
        lane = Lane(
            lane_id=str(raw["id"]),
            description=str(raw.get("description", "")),
            make_target=str(raw["make_target"]),
        )
        lanes[lane.lane_id] = lane
    presets = {str(k): [str(x) for x in v] for k, v in dict(cfg.get("presets", {})).items()}
    return lanes, presets


def _emit(payload: dict[str, Any], report_format: str) -> None:
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
        return
    if payload.get("action") == "list":
        for lane in payload.get("lanes", []):
            print(f"{lane['id']}: {lane['description']} (target={lane['make_target']})")
        return
    print(
        f"gates run: status={payload['status']} total={payload['total_count']} "
        f"failed={payload['failed_count']} run_id={payload['run_id']}"
    )
    for row in payload.get("results", []):
        if row["status"] == "fail":
            print(f"- FAIL {row['id']}: {row.get('error', 'failed')}")


def _run_one(repo_root: Path, lane: Lane) -> dict[str, Any]:
    result = run_command(["make", "-s", lane.make_target], repo_root)
    row: dict[str, Any] = {
        "id": lane.lane_id,
        "make_target": lane.make_target,
        "status": "pass" if result.code == 0 else "fail",
    }
    if result.code != 0:
        row["error"] = result.combined_output.splitlines()[-1] if result.combined_output else "lane failed"
    return row


def run_gates_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    lanes_by_id, presets = _load_lanes(ctx.repo_root)
    if ns.gates_cmd == "list":
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "pass",
            "action": "list",
            "lanes": [
                {"id": lane.lane_id, "description": lane.description, "make_target": lane.make_target}
                for lane in sorted(lanes_by_id.values(), key=lambda x: x.lane_id)
            ],
            "presets": presets,
        }
        _emit(payload, ns.report)
        return 0

    selected: list[str]
    single_lane = ns.lane or ns.lane_id
    if ns.all:
        selected = presets.get(ns.preset, [])
    elif single_lane:
        selected = [single_lane]
    else:
        selected = presets.get(ns.preset, [])
    lanes: list[Lane] = []
    missing = [lane_id for lane_id in selected if lane_id not in lanes_by_id]
    if missing:
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "fail",
            "action": "run",
            "run_id": ctx.run_id,
            "error": f"unknown lane ids: {', '.join(sorted(missing))}",
        }
        _emit(payload, ns.report)
        return 2
    for lane_id in selected:
        lanes.append(lanes_by_id[lane_id])

    results: list[dict[str, Any]] = []
    if ns.parallel and len(lanes) > 1:
        with ThreadPoolExecutor(max_workers=max(1, int(ns.jobs))) as pool:
            fut = {pool.submit(_run_one, ctx.repo_root, lane): lane for lane in lanes}
            for done in as_completed(fut):
                results.append(done.result())
    else:
        for lane in lanes:
            results.append(_run_one(ctx.repo_root, lane))

    ordered = sorted(results, key=lambda r: r["id"])
    failed = [r for r in ordered if r["status"] == "fail"]
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if failed else "pass",
        "action": "run",
        "run_id": ctx.run_id,
        "preset": ns.preset,
        "total_count": len(ordered),
        "failed_count": len(failed),
        "results": ordered,
    }
    out_json = ensure_evidence_path(ctx, ctx.evidence_root / "gates" / ctx.run_id / "report.json")
    out_txt = ensure_evidence_path(ctx, ctx.evidence_root / "gates" / ctx.run_id / "report.txt")
    write_text_file(out_json, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = [
        f"gates run: status={payload['status']} total={payload['total_count']} failed={payload['failed_count']} run_id={payload['run_id']}"
    ]
    lines.extend(f"- {row['status'].upper()} {row['id']} ({row['make_target']})" for row in ordered)
    write_text_file(out_txt, "\n".join(lines) + "\n", encoding="utf-8")
    payload["artifact_json"] = out_json.relative_to(ctx.repo_root).as_posix()
    payload["artifact_txt"] = out_txt.relative_to(ctx.repo_root).as_posix()
    _emit(payload, ns.report)
    return 1 if failed else 0


def configure_gates_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("gates", help="run curated gate lanes from SSOT manifest")
    gates_sub = p.add_subparsers(dest="gates_cmd", required=True)

    listing = gates_sub.add_parser("list", help="list configured lanes and presets")
    listing.add_argument("--report", choices=["text", "json"], default="text")

    run = gates_sub.add_parser("run", help="run one lane, a preset, or all lanes from preset")
    run.add_argument("lane_id", nargs="?", default="", help="optional positional lane id")
    run.add_argument("--lane", help="single lane id")
    run.add_argument("--preset", default="root", help="preset id from configs/gates/lanes.json")
    run.add_argument("--all", action="store_true", help="run all lanes from selected preset")
    run.add_argument("--parallel", action="store_true", help="run lanes in parallel")
    run.add_argument("--jobs", type=int, default=4)
    run.add_argument("--report", choices=["text", "json"], default="text")
