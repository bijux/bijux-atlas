from __future__ import annotations

import json
import subprocess
import tarfile
from datetime import datetime, timedelta, timezone
from pathlib import Path
from xml.etree.ElementTree import Element, SubElement, tostring

from ..core.context import RunContext
from ..checks.domains.policies.make.enforcement import collect_bypass_inventory
from .artifact_actions import _cmd_artifact_gc, _cmd_artifact_index

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


def _repo_cleanliness_report(ctx: RunContext) -> dict[str, object]:
    proc = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=ctx.repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    lines = [line for line in (proc.stdout or "").splitlines() if line.strip()]
    cache_markers = ("/__pycache__/", ".pytest_cache/", ".ruff_cache/", ".mypy_cache/", ".pyc")
    cache_paths: list[str] = []
    for line in lines:
        path = line[3:] if len(line) > 3 else line
        if any(marker in path for marker in cache_markers) or path.endswith(".pyc"):
            cache_paths.append(path)
    return {
        "status": "pass" if not lines else "warn",
        "git_dirty_path_count": len(lines),
        "python_cache_hygiene": {
            "status": "pass" if not cache_paths else "fail",
            "violations": sorted(set(cache_paths)),
        },
    }


def _ops_workflows_report(ctx: RunContext, run_id: str) -> dict[str, object]:
    root = ctx.repo_root / "artifacts" / "runs" / run_id / "ops"
    rows: list[dict[str, object]] = []
    if root.exists():
        for path in sorted(root.rglob("report.json")):
            try:
                payload = json.loads(path.read_text(encoding="utf-8"))
            except Exception:
                payload = {}
            rows.append(
                {
                    "path": str(path.relative_to(ctx.repo_root)),
                    "kind": str(payload.get("kind", "")),
                    "status": str(payload.get("status", "unknown")),
                }
            )
    statuses = {str(row.get("status", "")) for row in rows}
    status = "pass"
    if any(s in {"fail", "error"} for s in statuses):
        status = "fail"
    elif not rows:
        status = "warn"
    return {
        "status": status,
        "root": str(root.relative_to(ctx.repo_root)),
        "report_count": len(rows),
        "reports": rows,
    }


def _config_compilation_report(ctx: RunContext) -> dict[str, object]:
    root = ctx.repo_root / "configs" / "_generated"
    compiler = root / "compiler-report.json"
    checksums = root / "checksums.json"
    payload: dict[str, object] = {
        "status": "warn",
        "generated_root": "configs/_generated",
        "inventory_root": "ops/inventory",
        "files_present": [],
    }
    files_present = [p.name for p in (compiler, checksums) if p.exists()]
    payload["files_present"] = sorted(files_present)
    if compiler.exists():
        try:
            raw = compiler.read_text(encoding="utf-8")
            data = json.loads(raw.split("\n", 1)[1] if raw.startswith("# GENERATED") else raw)
            payload["inventory_root"] = str(data.get("inventory_root", "ops/inventory"))
            payload["overlay_model_status"] = "pass" if not ((data.get("overlay_model") or {}).get("errors")) else "fail"
        except Exception:
            payload["overlay_model_status"] = "invalid"
    if compiler.exists() and checksums.exists():
        payload["status"] = "pass"
    return payload


def _bypass_debt_report(ctx: RunContext) -> dict[str, object]:
    today = datetime.now(timezone.utc).date()
    inventory = collect_bypass_inventory(ctx.repo_root)
    rows = inventory.get("entries", []) if isinstance(inventory.get("entries"), list) else []
    overdue = 0
    due_14d = 0
    missing_required = 0
    for row in rows:
        if not isinstance(row, dict):
            continue
        if bool(row.get("requires_metadata")) and not all(
            str(row.get(field, "")).strip() for field in ("owner", "issue_id", "expiry", "scope", "removal_plan")
        ):
            missing_required += 1
        expiry = str(row.get("expiry", "")).strip()
        if not expiry:
            continue
        try:
            due = datetime.fromisoformat(expiry).date()
        except ValueError:
            continue
        if due < today:
            overdue += 1
        if due <= (today + timedelta(days=14)):
            due_14d += 1
    errors = inventory.get("errors", []) if isinstance(inventory.get("errors"), list) else []
    status = "pass"
    if overdue > 0 or missing_required > 0:
        status = "fail"
    elif due_14d > 0 or len(errors) > 0:
        status = "warn"
    return {
        "status": status,
        "entry_count": len(rows),
        "overdue_count": overdue,
        "due_within_14d_count": due_14d,
        "missing_required_metadata_count": missing_required,
        "inventory_error_count": len(errors),
    }


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

    product_artifacts: dict[str, object] | None = None
    product_manifest = ctx.evidence_root / "product" / "build" / run_id / "artifact-manifest.json"
    if product_manifest.exists():
        try:
            p = json.loads(product_manifest.read_text(encoding="utf-8"))
            artifacts = p.get("artifacts", [])
            product_artifacts = {
                "status": "ok",
                "manifest": str(product_manifest),
                "version": p.get("version", "unknown"),
                "artifact_count": len(artifacts) if isinstance(artifacts, list) else 0,
                "artifacts": artifacts if isinstance(artifacts, list) else [],
            }
        except Exception:
            product_artifacts = {"status": "invalid", "manifest": str(product_manifest)}

    payload = {
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
        "repo_cleanliness": _repo_cleanliness_report(ctx),
        "ops_workflows": _ops_workflows_report(ctx, run_id),
        "config_compilation": _config_compilation_report(ctx),
        "bypass_debt": _bypass_debt_report(ctx),
    }
    if product_artifacts is not None:
        payload["product_artifacts"] = product_artifacts
    return payload


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
    schema_path = ctx.repo_root / "ops/schema/report/unified.schema.json"
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
    out_path = Path(out) if out else (ctx.repo_root / "ops/_generated.example/scorecard.json")
    cmd = [
        "python3",
        "./packages/atlasctl/src/atlasctl/reporting/tools/make_confidence_scorecard.py",
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
    if proc.returncode == 0 and out_path.exists():
        try:
            scorecard = json.loads(out_path.read_text(encoding="utf-8"))
            bypass = collect_bypass_inventory(ctx.repo_root)
            entries = bypass.get("entries", []) if isinstance(bypass.get("entries"), list) else []
            oldest = None
            for row in entries:
                if not isinstance(row, dict):
                    continue
                created = str(row.get("created_at", "")).strip()
                if not created:
                    continue
                try:
                    age = (datetime.now(timezone.utc).date() - datetime.fromisoformat(created).date()).days
                except ValueError:
                    continue
                oldest = age if oldest is None else max(oldest, age)
            scorecard["bypass_inventory"] = {
                "entry_count": int(bypass.get("entry_count", 0)),
                "oldest_age_days": oldest,
                "error_count": len(bypass.get("errors", [])) if isinstance(bypass.get("errors"), list) else 0,
            }
            out_path.write_text(json.dumps(scorecard, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        except Exception:
            pass
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
