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
from ...checks.registry.ssot import generate_registry_json
from ...checks.core.execution import run_function_checks
from ...checks.engine.runner import domains as check_domains
from ...checks.engine.runner import run_domain
from ...contracts.ids import CHECK_RUN
from ...contracts.validate_self import validate_self
from ...core.context import RunContext
from ...core.fs import ensure_evidence_path
from ...core.runtime.paths import write_text_file
from ...core.runtime.telemetry import emit_telemetry
from ...core.exit_codes import ERR_USER
from ...commands.policies.lint.suite_engine import run_lint_suite

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
    code, payload = run_lint_suite(ctx.repo_root, suite_name, fail_fast=fail_fast)
    if ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"check {label}: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
    return code


def _parse_select(value: str) -> tuple[str | None, str]:
    if not value:
        return None, ""
    if value.startswith("atlasctl::"):
        parts = value.split("::")
        if len(parts) == 3:
            _, domain, selector = parts
            return domain.strip() or None, selector.strip()
    return None, value


def _split_marker_values(raw_values: list[str] | None) -> set[str]:
    values: set[str] = set()
    for raw in raw_values or []:
        for part in str(raw).split(","):
            marker = part.strip()
            if marker:
                values.add(marker)
    return values


def _split_group_values(raw: list[str] | None) -> set[str]:
    values: set[str] = set()
    for item in raw or []:
        for part in str(item).split(","):
            value = part.strip()
            if value:
                values.add(value)
    return values


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
    return (
        selector == check_id
        or fnmatch(check_id, selector)
        or selector in check_id
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
    checks = [check for check in list_checks() if selected_domain is None or check.domain == selected_domain]
    group = str(getattr(ns, "group", "") or "").strip()
    if domain_value and not group:
        group = domain_value
    if group and group != "all":
        if group.endswith("-slow"):
            base = group.removesuffix("-slow")
            checks = [check for check in checks if check.domain == base and check.slow]
        elif group.endswith("-fast"):
            base = group.removesuffix("-fast")
            checks = [check for check in checks if check.domain == base and not check.slow]
        else:
            checks = [check for check in checks if (check.domain == group or group in set(check_tags(check)))]
    exclude_groups = _split_group_values(getattr(ns, "exclude_group", []))
    if exclude_groups:
        kept = []
        for check in checks:
            groups = set(check_tags(check))
            groups.add(check.domain)
            if groups.intersection(exclude_groups):
                continue
            kept.append(check)
        checks = kept
    only_slow = bool(getattr(ns, "only_slow", False))
    only_fast = bool(getattr(ns, "only_fast", False))
    if only_slow and only_fast:
        print("invalid selection: --slow and --fast cannot be used together")
        return ERR_USER
    if only_slow:
        checks = [check for check in checks if check.slow]
    if only_fast:
        checks = [check for check in checks if not check.slow]
    include_all = bool(getattr(ns, "include_all", False))
    explicit_selector = bool(id_value or target_value or select_value or k_value)
    if explicit_selector:
        include_all = True
    marker_values = _split_marker_values(getattr(ns, "marker", []))
    if "slow" in marker_values:
        include_all = True
    if not include_all and not only_slow and not group.endswith("-slow"):
        checks = [check for check in checks if not check.slow]
    match_pattern = str(getattr(ns, "match", "") or "").strip()
    if match_pattern:
        checks = [
            check
            for check in checks
            if fnmatch(check.check_id, match_pattern) or fnmatch(check.title, match_pattern) or fnmatch(f"atlasctl::{check.domain}::{check.check_id}", match_pattern)
        ]
    required_markers = _split_marker_values(getattr(ns, "require_markers", []))
    required_markers |= marker_values
    if required_markers:
        checks = [check for check in checks if required_markers.issubset(set(check_tags(check)))]
    excluded_markers = _split_marker_values(getattr(ns, "exclude_marker", []))
    if excluded_markers:
        checks = [check for check in checks if not set(check_tags(check)).intersection(excluded_markers)]
    matched_checks = [
        check for check in checks if _match_selected(check.check_id, check.title, check.domain, selected_domain, selector)
    ]
    matched_checks = sorted(matched_checks, key=lambda item: item.check_id)
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
        row_status = "PASS" if result.status == "pass" else "FAIL"
        row_duration = int(result.metrics.get("duration_ms", 0))
        if ns.run_quiet:
            print(_format_progress_line(live_index, total_live, result.id, row_status))
            return
        if ns.run_verbose:
            owners = ",".join(result.owners) if result.owners else "-"
            print(_format_progress_line(live_index, total_live, result.id, f"{row_status} ({row_duration}ms)"))
            print(f"  owners={owners} hint={result.fix_hint}")
            if row_status == "FAIL" and result.errors:
                print(f"  detail: {result.errors[0]}")
            return
        print(_format_progress_line(live_index, total_live, result.id, row_status))

    if live_print:
        timeout_label = "disabled" if timeout_ms == 0 else f"{timeout_ms}ms"
        print(f"running {total_live} checks (timeout={timeout_label} per check)")

    jobs = max(1, int(getattr(ns, "jobs", 1) or 1))
    max_failures = int(getattr(ns, "max_failures", 0) or getattr(ns, "maxfail", 0) or 0)
    if bool(getattr(ns, "failfast", False)):
        max_failures = 1
    executed_results = []
    if max_failures > 0 and jobs == 1:
        fail_seen = 0
        for check in matched_checks:
            _failed, one = run_function_checks(
                ctx.repo_root,
                [check],
                on_result=_emit_live_row if live_print else None,
                timeout_ms=timeout_ms if timeout_ms > 0 else None,
                jobs=1,
            )
            executed_results.extend(one)
            if _failed:
                fail_seen += _failed
                if fail_seen >= max_failures and not bool(getattr(ns, "keep_going", False)):
                    break
    else:
        _failed_total, executed_results = run_function_checks(
            ctx.repo_root,
            matched_checks,
            on_result=_emit_live_row if live_print else None,
            timeout_ms=timeout_ms if timeout_ms > 0 else None,
            jobs=jobs,
        )
    executed_by_id = {result.id: result for result in executed_results}
    rows: list[dict[str, object]] = []
    fail_count = 0
    for check in report_checks:
        result = executed_by_id.get(check.check_id)
        if result is None:
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
                }
            )
            continue
        status = "PASS" if result.status == "pass" else "FAIL"
        detail_errors = list(result.errors) + [f"WARN: {warn}" for warn in result.warnings]
        detail = "; ".join(detail_errors[:2]) if detail_errors else ""
        rows.append(
            {
                "id": check.check_id,
                "title": check.title,
                "domain": check.domain,
                "status": status,
                "duration_ms": int(result.metrics.get("duration_ms", 0)),
                "reason": detail if detail else ("check failed" if status == "FAIL" else ""),
                "hints": [check.fix_hint] if check.fix_hint else [],
                "hint": check.fix_hint,
                "detail": detail,
                "owners": list(check.owners),
                "artifacts": list(result.evidence_paths),
            }
        )
        if status == "FAIL":
            fail_count += 1

    total_duration_ms = int((time.perf_counter() - started) * 1000)
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
    final_failed = fail_count + (1 if ratchet_errors else 0) + (1 if speed_regressions else 0)
    summary = {
        "passed": pass_count,
        "failed": fail_count,
        "skipped": skip_count,
        "total": len(rows),
        "duration_ms": total_duration_ms,
    }

    if ctx.output_format == "json" or ns.json:
        payload = {
            "schema_name": CHECK_RUN,
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "check-run",
            "run_id": ctx.run_id,
            "status": "ok" if final_failed == 0 else "error",
            "summary": summary,
            "slow_threshold_ms": slow_threshold_ms,
            "slow_checks": slow_rows,
            "ratchet_errors": ratchet_errors,
            "speed_regressions": speed_regressions,
            "rows": rows,
            "timing_histogram": _timing_histogram(rows),
        }
        validate_self(CHECK_RUN, payload)
        print(json.dumps(payload, sort_keys=True))
    elif bool(getattr(ns, "jsonl", False)):
        for row in rows:
            print(json.dumps({"kind": "check-row", **row}, sort_keys=True))
        print(
            json.dumps(
                {
                    "kind": "summary",
                    "summary": summary,
                    "slow_threshold_ms": slow_threshold_ms,
                    "slow_checks": slow_rows,
                    "ratchet_errors": ratchet_errors,
                },
                sort_keys=True,
            )
        )
    else:
        if not live_print:
            for row in rows:
                if ns.run_quiet:
                    print(f"{row['status']} {row['id']}")
                    continue
                if ns.run_verbose:
                    owners = ",".join(row["owners"]) if row["owners"] else "-"
                    print(f"{row['status']} {row['id']} [{row['duration_ms']}ms] owners={owners} hint={row['hint']}")
                    if row["status"] == "FAIL" and row["detail"]:
                        print(f"  detail: {row['detail']}")
                    continue
                print(f"{row['status']} {row['id']} ({row['duration_ms']}ms)")
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

    if ns.json_report:
        report_path = ensure_evidence_path(ctx, Path(ns.json_report))
        report_payload = {
            "schema_name": CHECK_RUN,
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "check-run-report",
            "run_id": ctx.run_id,
            "status": "ok" if final_failed == 0 else "error",
            "summary": summary,
            "slow_threshold_ms": slow_threshold_ms,
            "slow_checks": slow_rows,
            "ratchet_errors": ratchet_errors,
            "rows": rows,
        }
        validate_self(CHECK_RUN, report_payload)
        write_text_file(report_path, json.dumps(report_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    slow_report = getattr(ns, "slow_report", None)
    if slow_report:
        slow_path = ensure_evidence_path(ctx, Path(slow_report))
        write_text_file(
            slow_path,
            json.dumps(
                {
                    "schema_version": 1,
                    "tool": "atlasctl",
                    "kind": "check-slow-report",
                    "run_id": ctx.run_id,
                    "threshold_ms": slow_threshold_ms,
                    "summary": summary,
                    "slow_checks": slow_rows,
                    "ratchet_errors": ratchet_errors,
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
    if ns.profile:
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
    timings_path = ctx.repo_root / "artifacts" / "evidence" / "checks" / ctx.run_id / "timings.json"
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
