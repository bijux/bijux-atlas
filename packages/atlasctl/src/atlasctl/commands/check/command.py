from __future__ import annotations

import argparse
import inspect
import json
import subprocess
import time
from fnmatch import fnmatch
from pathlib import Path
from typing import Callable
from xml.etree.ElementTree import Element, SubElement, tostring

from ...checks.registry import check_tags, get_check, list_checks
from ...checks.report import build_report_payload, render_json, render_jsonl, render_text
from ...checks.selectors import apply_selection_criteria, parse_selection_criteria
from ...checks.registry import generate_registry_json
from ...registry.suites import suite_manifest_specs, resolve_check_ids
from ...engine.runner import domains as check_domains
from ...engine.runner import run_domain
from ...checks.tools.repo_domain.enforcement.package_shape import check_module_size
from ...checks.tools.repo_domain.native.modules.repo_checks_make_and_layout import check_layout_contract
from ...core.context import RunContext
from ...core.fs import ensure_evidence_path
from ...core.runtime.paths import write_text_file
from ...core.runtime.telemetry import emit_telemetry
from ...core.exit_codes import ERR_USER
from ...engine.runner import RunnerOptions, run_checks_payload
from ...checks.effects import CheckEffect, normalize_effect
from ...checks.runner import report_from_payload
from .selection import split_group_values, split_marker_values

NativeCheck = Callable[[Path], tuple[int, list[str]]]
SHELL_POLICY_CHECK_IDS: tuple[str, ...] = (
    "checks_repo_shell_location_policy",
    "checks_repo_shell_strict_mode",
    "checks_repo_shell_no_direct_python",
    "checks_repo_shell_no_network_fetch",
    "checks_repo_shell_invocation_boundary",
    "checks_repo_core_no_bash_subprocess",
    "checks_repo_shell_readonly_checks",
    "checks_repo_shell_script_budget",
    "checks_repo_shell_docs_present",
)
CHECK_EVIDENCE_ROOT = "artifacts/atlasctl/checks"


def _run(ctx: RunContext, cmd: list[str]) -> int:
    return subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False).returncode


def _print_errors(title: str, errors: list[str], ok_message: str, limit: int | None = None, prefix: str = "- ") -> None:
    if errors:
        print(title)
        for err in errors[:limit] if limit else errors:
            print(f"{prefix}{err}")
    else:
        print(ok_message)


def _run_native_check(ctx: RunContext, fn: NativeCheck, title: str, ok_message: str, limit: int | None = None, prefix: str = "- ") -> int:
    code, errors = fn(ctx.repo_root)
    _print_errors(title, errors, ok_message, limit=limit, prefix=prefix)
    return code


def _run_domain(ctx: RunContext, domain: str, fail_fast: bool = False, label: str | None = None) -> int:
    code, payload = run_domain(ctx.repo_root, domain, fail_fast=fail_fast)
    if ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        name = label or domain
        print(f"check {name}: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
    return code


def _run_suite_domain(ctx: RunContext, suite_name: str, label: str, fail_fast: bool) -> int:
    return _run_domain(ctx, suite_name, fail_fast=fail_fast, label=label)


def _parse_select(value: str) -> tuple[str | None, str]:
    if not value:
        return None, ""
    if value.startswith("atlasctl::"):
        parts = value.split("::")
        if len(parts) == 3:
            _, domain, selector = parts
            return domain.strip() or None, selector.strip()
    return None, value


def _check_source_path(ctx: RunContext, check_id: str) -> str | None:
    check = get_check(check_id)
    if check is None:
        return None
    source = inspect.getsourcefile(check.fn)
    if not source:
        return None
    path = Path(source).resolve()
    try:
        return path.relative_to(ctx.repo_root).as_posix()
    except ValueError:
        return path.as_posix()


def _match_selected(check_id: str, title: str, domain: str, selected_domain: str | None, selector: str) -> bool:
    if selected_domain and domain != selected_domain:
        return False
    if not selector:
        return True
    canonical_alias = check_id if check_id.startswith("checks_") else f"checks_{check_id.replace('.', '_')}"
    return (
        selector == check_id
        or selector == canonical_alias
        or fnmatch(check_id, selector)
        or fnmatch(canonical_alias, selector)
        or selector in check_id
        or selector in canonical_alias
        or selector in title
    )


def _write_junitxml(path: Path, rows: list[dict[str, object]]) -> None:
    suite = Element("testsuite", name="atlasctl-check-run", tests=str(len(rows)), failures=str(sum(1 for row in rows if row["status"] == "FAIL")))
    for row in rows:
        case = SubElement(suite, "testcase", classname=f"atlasctl.checks.{row['domain']}", name=str(row["id"]), time=f"{float(row['duration_ms']) / 1000.0:.6f}")
        if row["status"] == "FAIL":
            failure = SubElement(case, "failure", message=str(row.get("hint", "failed")))
            failure.text = str(row.get("detail", "check failed"))
        if row["status"] == "SKIP":
            skipped = SubElement(case, "skipped", message="filtered by --select")
            skipped.text = "filtered by --select"
    write_text_file(path, tostring(suite, encoding="unicode"), encoding="utf-8")


def _run_check_registry(ctx: RunContext, ns: argparse.Namespace) -> int:
    started = time.perf_counter()
    select_value = str(getattr(ns, "select", "") or "").strip()
    id_value = str(getattr(ns, "id", "") or "").strip()
    k_value = str(getattr(ns, "k", "") or "").strip()
    target_value = str(getattr(ns, "check_target", "") or "").strip()
    domain_value = str(getattr(ns, "domain_filter", "") or "").strip()
    suite_name = str(getattr(ns, "suite", "") or "").strip()
    if id_value and not select_value:
        select_value = id_value
    if k_value and not select_value:
        select_value = k_value
    if target_value and not select_value:
        select_value = target_value
    if "." in id_value and not bool(getattr(ns, "legacy_id", False)) and not (ctx.output_format == "json" or ns.json):
        print("warning: using legacy dotted check id without --legacy-id; migrate to canonical checks_<domain>_<area>_<name>")
    selected_domain, selector = _parse_select(select_value)
    domain_aliases = set(check_domains())
    if target_value in domain_aliases:
        if target_value != "all" and not str(getattr(ns, "group", "") or "").strip():
            setattr(ns, "group", target_value)
        selected_domain = None
        selector = ""
    checks = list(list_checks())
    selection_profile = str(getattr(ns, "profile", "") or "").strip()
    if selection_profile:
        if selection_profile == "slow":
            setattr(ns, "only_slow", True)
            setattr(ns, "only_fast", False)
            setattr(ns, "include_all", False)
        elif selection_profile == "fast":
            setattr(ns, "only_fast", True)
            setattr(ns, "only_slow", False)
            setattr(ns, "include_all", False)
        elif selection_profile == "all":
            setattr(ns, "include_all", True)
            setattr(ns, "only_slow", False)
            setattr(ns, "only_fast", False)
    category = str(getattr(ns, "category", "") or "").strip()
    if category:
        checks = [check for check in checks if str(check.category) == category]
    if suite_name:
        spec = next((item for item in suite_manifest_specs() if item.name == suite_name), None)
        if spec is None:
            print(f"unknown suite: {suite_name}")
            return ERR_USER
        suite_ids = set(resolve_check_ids(spec))
        checks = [check for check in checks if check.check_id in suite_ids]
    setattr(ns, "domain_filter", selected_domain or domain_value or str(getattr(ns, "domain_filter", "") or "").strip())
    if bool(getattr(ns, "only_slow", False)) and bool(getattr(ns, "only_fast", False)):
        print("invalid selection: --slow and --fast cannot be used together")
        return ERR_USER
    criteria = parse_selection_criteria(ns, ctx.repo_root)
    checks = apply_selection_criteria(checks, criteria)
    group = str(getattr(ns, "group", "") or "").strip()
    if group and group != "all":
        checks = [check for check in checks if (str(check.domain) == group or group in set(check_tags(check)))]
    exclude_groups = split_group_values(getattr(ns, "exclude_group", []))
    if exclude_groups:
        checks = [
            check
            for check in checks
            if not ({str(check.domain), *set(check_tags(check))}.intersection(exclude_groups))
        ]
    marker_values = split_marker_values(getattr(ns, "marker", []))
    match_pattern = str(getattr(ns, "match", "") or "").strip()
    if match_pattern:
        checks = [
            check
            for check in checks
            if fnmatch(check.check_id, match_pattern) or fnmatch(check.title, match_pattern) or fnmatch(f"atlasctl::{check.domain}::{check.check_id}", match_pattern)
        ]
    required_markers = split_marker_values(getattr(ns, "require_markers", []))
    required_markers |= marker_values
    if required_markers:
        checks = [check for check in checks if required_markers.issubset(set(check_tags(check)))]
    excluded_markers = split_marker_values(getattr(ns, "exclude_marker", []))
    if excluded_markers:
        checks = [check for check in checks if not set(check_tags(check)).intersection(excluded_markers)]
    matched_checks = [check for check in checks if _match_selected(check.check_id, check.title, check.domain, selected_domain, selector)]
    matched_checks = sorted(matched_checks, key=lambda item: item.check_id)
    explicit_selector = bool(id_value)
    if explicit_selector and not matched_checks:
        query = select_value or id_value or k_value or target_value or match_pattern
        print(f"no checks matched selector: {query}")
        return ERR_USER
    report_checks = matched_checks if selector else checks
    report_checks = sorted(report_checks, key=lambda item: item.check_id)
    if bool(getattr(ns, "list_selected", False)):
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "check-selection",
            "status": "ok",
            "count": len(report_checks),
            "checks": [check.check_id for check in report_checks],
        }
        if ctx.output_format == "json" or ns.json:
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"selected={payload['count']}")
            for check_id in payload["checks"]:
                print(check_id)
        return 0
    live_print = bool(matched_checks) and not (ctx.output_format == "json" or ns.json or bool(getattr(ns, "jsonl", False)))
    timeout_ms = max(0, int(getattr(ns, "timeout_ms", 2000) or 0))
    if timeout_ms and timeout_ms < 50:
        print("invalid --timeout-ms: minimum is 50ms (or 0 to disable)")
        return ERR_USER
    total_live = len(matched_checks)
    live_index = 0

    def _format_progress_line(index: int, total: int, check_id: str, status: str) -> str:
        prefix = f"[{index}/{total}] {check_id} "
        suffix = status
        width = 110
        dots = "." * max(8, width - len(prefix) - len(suffix) - 1)
        return f"{prefix}{dots} {suffix}"

    def _emit_live_row(result):  # noqa: ANN001
        nonlocal live_index
        live_index += 1
        row_status_raw = str(getattr(result, "status", "fail")).lower()
        if row_status_raw == "pass":
            row_status = "PASS"
        elif row_status_raw == "skip":
            row_status = "SKIP"
        else:
            row_status = "FAIL"
        if row_status == "SKIP" and not bool(getattr(ns, "show_skips", False)):
            return
        metrics = getattr(result, "metrics", {}) or {}
        if isinstance(metrics, dict):
            row_duration = int(metrics.get("duration_ms", 0))
        else:
            row_duration = int(getattr(result, "duration_ms", 0))
        row_id = str(getattr(result, "id", getattr(result, "check_id", "unknown-check")))
        if ns.run_quiet:
            print(_format_progress_line(live_index, total_live, row_id, row_status))
            return
        if ns.run_verbose:
            owners_val = getattr(result, "owners", ())
            owners = ",".join(owners_val) if owners_val else "-"
            print(_format_progress_line(live_index, total_live, row_id, f"{row_status} ({row_duration}ms)"))
            print(f"  owners={owners} hint={getattr(result, 'fix_hint', '-')}")
            errors_val = getattr(result, "errors", ())
            if row_status == "FAIL" and errors_val:
                print(f"  detail: {list(errors_val)[0]}")
            return
        print(_format_progress_line(live_index, total_live, row_id, row_status))

    if live_print:
        timeout_label = "disabled" if timeout_ms == 0 else f"{timeout_ms}ms"
        print(f"running {total_live} checks (timeout={timeout_label} per check)")

    requires_write_root = any(CheckEffect.FS_WRITE.value in {normalize_effect(x) for x in check.effects} for check in matched_checks)
    write_root_arg = str(getattr(ns, "write_root", "") or "").strip()
    if requires_write_root and not write_root_arg:
        print("write-enabled checks require --write-root under artifacts/runs/<run_id>/")
        return ERR_USER
    run_evidence_root = ensure_evidence_path(ctx, Path(f"{CHECK_EVIDENCE_ROOT}/{ctx.run_id}"))
    resolved_run_root = run_evidence_root.resolve()
    if write_root_arg:
        candidate = Path(write_root_arg)
        resolved_run_root = (candidate if candidate.is_absolute() else (ctx.repo_root / candidate)).resolve()
        try:
            rel = resolved_run_root.relative_to(ctx.repo_root).as_posix()
        except ValueError:
            print("--write-root must be inside repository")
            return ERR_USER
        if requires_write_root and not rel.startswith(f"artifacts/runs/{ctx.run_id}/"):
            print(f"--write-root must be under artifacts/runs/{ctx.run_id}/ for write-enabled runs")
            return ERR_USER

    jobs = max(1, int(getattr(ns, "jobs", 1) or 1))
    max_failures = int(getattr(ns, "max_failures", 0) or getattr(ns, "maxfail", 0) or 0)
    if bool(getattr(ns, "failfast", False)):
        max_failures = 1
    runner_rc, runner_payload = run_checks_payload(
        ctx.repo_root,
        check_defs=matched_checks,
        run_id=ctx.run_id,
        options=RunnerOptions(
            fail_fast=bool(max_failures == 1 and jobs == 1 and not bool(getattr(ns, "keep_going", False))),
            jobs=jobs,
            timeout_ms=timeout_ms if timeout_ms > 0 else None,
            output="json" if (ctx.output_format == "json" or ns.json) else ("verbose" if ns.run_verbose else ("quiet" if ns.run_quiet else "text")),
            kind="check-run",
            run_root=resolved_run_root,
        ),
        on_event=_emit_live_row if live_print else None,
    )
    executed_by_id = {str(row["id"]): row for row in runner_payload.get("rows", []) if isinstance(row, dict)}
    rows: list[dict[str, object]] = []
    for check in report_checks:
        runner_row = executed_by_id.get(check.check_id)
        if runner_row is None:
            rows.append(
                {
                    "id": check.check_id,
                    "title": check.title,
                    "domain": check.domain,
                    "status": "SKIP",
                    "duration_ms": 0,
                    "reason": "filtered by --select",
                    "hints": ["Adjust --select/--group/--marker filters to include this check."],
                    "hint": "filtered by --select",
                    "detail": "",
                    "owners": list(check.owners),
                    "artifacts": [],
                    "findings": [],
                    "category": check.category.value,
                }
            )
            continue
        rows.append(
            {
                "id": check.check_id,
                "title": check.title,
                "domain": check.domain,
                "status": str(runner_row.get("status", "FAIL")),
                "duration_ms": int(runner_row.get("duration_ms", 0)),
                "reason": str(runner_row.get("reason", "")),
                "hints": list(runner_row.get("hints", [])),
                "hint": (list(runner_row.get("hints", []))[:1] or [check.fix_hint])[0],
                "detail": str(runner_row.get("reason", "")),
                "owners": list(runner_row.get("owners", [])),
                "artifacts": list(runner_row.get("artifacts", [])),
                "findings": list(runner_row.get("findings", [])),
                "category": str(runner_row.get("category", "check")),
                "attachments": list(runner_row.get("attachments", [])),
            }
        )
    fail_count = sum(1 for row in rows if row["status"] == "FAIL")

    total_duration_ms = 0 if not rows else int((time.perf_counter() - started) * 1000)
    pass_count = sum(1 for row in rows if row["status"] == "PASS")
    skip_count = sum(1 for row in rows if row["status"] == "SKIP")
    slow_threshold_ms = max(1, int(getattr(ns, "slow_threshold_ms", 800)))
    slow_rows = sorted(
        [row for row in rows if row["status"] != "SKIP" and int(row["duration_ms"]) >= slow_threshold_ms],
        key=lambda item: int(item["duration_ms"]),
        reverse=True,
    )
    ratchet_errors: list[str] = []
    ratchet_path = Path(getattr(ns, "slow_ratchet_config", "configs/policy/slow-checks-ratchet.json"))
    if ratchet_path.exists():
        ratchet = json.loads(ratchet_path.read_text(encoding="utf-8"))
        max_slow_checks = int(ratchet.get("max_slow_checks", 0))
        max_slowest_ms = int(ratchet.get("max_slowest_ms", 0))
        if max_slow_checks and len(slow_rows) > max_slow_checks:
            ratchet_errors.append(f"slow checks ratchet exceeded: {len(slow_rows)} > {max_slow_checks}")
        if max_slowest_ms and slow_rows and int(slow_rows[0]["duration_ms"]) > max_slowest_ms:
            ratchet_errors.append(f"slowest check ratchet exceeded: {int(slow_rows[0]['duration_ms'])}ms > {max_slowest_ms}ms")

    approvals_path = Path("configs/policy/check_speed_approvals.json")
    approvals: dict[str, int] = {}
    if approvals_path.exists():
        try:
            payload = json.loads((ctx.repo_root / approvals_path).read_text(encoding="utf-8"))
            approvals = {str(k): int(v) for k, v in payload.get("checks", {}).items()}
        except Exception:
            approvals = {}
    speed_regressions: list[str] = []
    for row in slow_rows:
        cid = str(row["id"])
        approved = approvals.get(cid)
        if approved is None:
            continue
        if int(row["duration_ms"]) > approved:
            speed_regressions.append(f"{cid}: {int(row['duration_ms'])}ms > approved {approved}ms")
    ignore_speed_regressions = bool(getattr(ns, "ignore_speed_regressions", False))
    speed_regression_failure = bool(speed_regressions) and not ignore_speed_regressions
    final_failed = fail_count + (1 if ratchet_errors else 0) + (1 if speed_regression_failure else 0)
    summary = {
        "passed": pass_count,
        "failed": fail_count,
        "skipped": skip_count,
        "total": len(rows),
        "duration_ms": total_duration_ms,
    }

    report_payload = build_report_payload(
        report_from_payload(
            {
                "summary": summary,
                "rows": rows,
            }
        ),
        run_id=ctx.run_id,
        slow_threshold_ms=slow_threshold_ms,
        slow_checks=slow_rows,
        ratchet_errors=ratchet_errors,
        speed_regressions=speed_regressions,
        events=list(runner_payload.get("events", [])),
        attachments=list(runner_payload.get("attachments", [])),
        timing_histogram=_timing_histogram(rows),
    )
    report_payload["status"] = "ok" if final_failed == 0 else "error"
    if ctx.output_format == "json" or ns.json:
        print(render_json(report_payload))
    elif bool(getattr(ns, "jsonl", False)):
        print(render_jsonl(report_payload))
    else:
        if not live_print:
            print(
                render_text(
                    report_payload,
                    quiet=bool(ns.run_quiet),
                    verbose=bool(ns.run_verbose),
                    show_skips=bool(getattr(ns, "show_skips", False)),
                )
            )
        if ns.durations and ns.durations > 0:
            print("durations:")
            ranked = sorted(rows, key=lambda item: int(item["duration_ms"]), reverse=True)[: ns.durations]
            for row in ranked:
                print(f"- {row['id']}: {row['duration_ms']}ms")
        print(
            f"summary: passed={pass_count} failed={fail_count} skipped={skip_count} "
            f"total={len(rows)} duration_ms={total_duration_ms}"
        )
        if slow_rows:
            print(f"slow checks (threshold={slow_threshold_ms}ms):")
            for row in slow_rows[:10]:
                print(f"- {row['id']}: {row['duration_ms']}ms")
        for msg in ratchet_errors:
            print(f"ratchet: {msg}")
        for msg in speed_regressions:
            print(f"speed-regression: {msg}")
        if fail_count:
            print("failing checks:")
            for row in rows:
                if row["status"] == "FAIL":
                    print(f"- {row['id']}: {row['detail'] or row['hint']}")

    default_report_path = ensure_evidence_path(ctx, Path(f"{CHECK_EVIDENCE_ROOT}/{ctx.run_id}/report.unified.json"))
    write_text_file(default_report_path, json.dumps(report_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if ns.json_report:
        report_path = ensure_evidence_path(ctx, Path(ns.json_report))
        if report_path != default_report_path:
            write_text_file(report_path, json.dumps(report_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    slow_report = getattr(ns, "slow_report", None)
    if slow_report:
        slow_path = ensure_evidence_path(ctx, Path(slow_report))
        write_text_file(
            slow_path,
            json.dumps(slow_rows, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    if bool(getattr(ns, "emit_profile", False)):
        profile_path = ensure_evidence_path(
            ctx,
            Path(getattr(ns, "profile_out", f"artifacts/isolate/{ctx.run_id}/atlasctl-check/profile.json")),
        )
        write_text_file(
            profile_path,
            json.dumps(
                {
                    "schema_version": 1,
                    "tool": "atlasctl",
                    "kind": "check-profile",
                    "run_id": ctx.run_id,
                    "summary": summary,
                    "rows": rows,
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
    timings_path = ensure_evidence_path(ctx, Path(f"{CHECK_EVIDENCE_ROOT}/{ctx.run_id}/timings.json"))
    timings_payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "check-timings",
        "run_id": ctx.run_id,
        "rows": [{"id": row["id"], "domain": row["domain"], "duration_ms": row["duration_ms"]} for row in rows],
        "timing_histogram": _timing_histogram(rows),
    }
    write_text_file(timings_path, json.dumps(timings_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    emit_telemetry(
        ctx,
        "check.run",
        passed=pass_count,
        failed=fail_count,
        skipped=skip_count,
        duration_ms=total_duration_ms,
        slow_checks=len(slow_rows),
    )
    junit_out = getattr(ns, "junit_xml", None) or getattr(ns, "junitxml", None)
    if junit_out:
        junit_path = ensure_evidence_path(ctx, Path(junit_out))
        _write_junitxml(junit_path, rows)
    return 0 if final_failed == 0 else ERR_USER


def _timing_histogram(rows: list[dict[str, object]]) -> dict[str, int]:
    buckets = {
        "lt_100ms": 0,
        "100_500ms": 0,
        "500_1000ms": 0,
        "1000_2000ms": 0,
        "gte_2000ms": 0,
    }
    for row in rows:
        if row.get("status") == "SKIP":
            continue
        d = int(row.get("duration_ms", 0))
        if d < 100:
            buckets["lt_100ms"] += 1
        elif d < 500:
            buckets["100_500ms"] += 1
        elif d < 1000:
            buckets["500_1000ms"] += 1
        elif d < 2000:
            buckets["1000_2000ms"] += 1
        else:
            buckets["gte_2000ms"] += 1
    return buckets



def run_check_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    from .run import run

    return run(ctx, ns)


def configure_check_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    from .parser import configure_check_parser as configure

    configure(sub)


def configure_checks_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    from .parser import configure_checks_parser as configure

    configure(sub)
