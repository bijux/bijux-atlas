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
from ...checks.execution import run_function_checks
from ...contracts.ids import CHECK_LIST
from ...contracts.validate_self import validate_self
from ...checks.repo.contracts.command_contracts import runtime_contracts_payload
from ...checks.repo.native import (
    check_atlas_scripts_cli_contract,
    check_atlasctl_boundaries,
    check_bin_entrypoints,
    check_committed_generated_hygiene,
    check_docs_scripts_references,
    check_duplicate_script_names,
    check_effects_lint,
    check_forbidden_top_dirs,
    check_invocation_parity,
    check_layout_contract,
    check_make_command_allowlist,
    check_make_forbidden_paths,
    check_make_help,
    check_make_scripts_references,
    check_naming_intent_lint,
    check_no_adhoc_python,
    check_no_direct_bash_invocations,
    check_no_direct_python_invocations,
    check_no_executable_python_outside_packages,
    check_no_xtask_refs,
    check_ops_generated_tracked,
    check_python_lock,
    check_python_migration_exceptions_expiry,
    check_python_runtime_artifacts,
    check_repo_script_boundaries,
    check_root_bin_shims,
    check_script_errors,
    check_script_help,
    check_script_ownership,
    check_script_shim_expiry,
    check_script_shims_minimal,
    check_script_tool_guards,
    check_script_write_roots,
    check_scripts_lock_sync,
    check_scripts_surface_docs_drift,
    check_tracked_timestamp_paths,
    check_venv_location_policy,
    generate_scripts_sbom,
)
from ...checks.repo.enforcement.module_size import check_module_size
from ...checks.runner import domains as check_domains
from ...checks.runner import run_domain
from ...core.context import RunContext
from ...core.fs import ensure_evidence_path
from ...core.telemetry import emit_telemetry
from ...core.exit_codes import ERR_CONTRACT, ERR_USER
from ...commands.policies.lint.suite_engine import run_lint_suite

NativeCheck = Callable[[Path], tuple[int, list[str]]]
SHELL_POLICY_CHECK_IDS: tuple[str, ...] = (
    "repo.shell_location_policy",
    "repo.shell_strict_mode",
    "repo.shell_no_direct_python",
    "repo.shell_no_network_fetch",
    "repo.shell_invocation_boundary",
    "repo.core_no_bash_subprocess",
    "repo.shell_readonly_checks",
    "repo.shell_script_budget",
    "repo.shell_docs_present",
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
    return selector == check_id or selector in check_id or selector in title


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
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(tostring(suite, encoding="unicode"), encoding="utf-8")


def _run_check_registry(ctx: RunContext, ns: argparse.Namespace) -> int:
    started = time.perf_counter()
    select_value = str(getattr(ns, "select", "") or "").strip()
    target_value = str(getattr(ns, "check_target", "") or "").strip()
    if target_value and not select_value:
        select_value = target_value
    selected_domain, selector = _parse_select(select_value)
    checks = [check for check in list_checks() if selected_domain is None or check.domain == selected_domain]
    group = str(getattr(ns, "group", "") or "").strip()
    if group:
        checks = [check for check in checks if check.domain == group]
    match_pattern = str(getattr(ns, "match", "") or "").strip()
    if match_pattern:
        checks = [
            check
            for check in checks
            if fnmatch(check.check_id, match_pattern) or fnmatch(check.title, match_pattern) or fnmatch(f"atlasctl::{check.domain}::{check.check_id}", match_pattern)
        ]
    required_markers = _split_marker_values(getattr(ns, "require_markers", []))
    if required_markers:
        checks = [check for check in checks if required_markers.issubset(set(check_tags(check)))]
    matched_checks = [
        check for check in checks if _match_selected(check.check_id, check.title, check.domain, selected_domain, selector)
    ]
    _failed_total, executed_results = run_function_checks(ctx.repo_root, matched_checks)
    executed_by_id = {result.id: result for result in executed_results}
    rows: list[dict[str, object]] = []
    fail_count = 0
    maxfail = 1 if ns.failfast else max(0, int(ns.maxfail or 0))
    for check in checks:
        result = executed_by_id.get(check.check_id)
        if result is None:
            rows.append(
                {
                    "id": check.check_id,
                    "title": check.title,
                    "domain": check.domain,
                    "status": "SKIP",
                    "duration_ms": 0,
                    "hint": "filtered by --select",
                    "detail": "",
                    "owners": list(check.owners),
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
                "hint": check.fix_hint,
                "detail": detail,
                "owners": list(check.owners),
            }
        )
        if status == "FAIL":
            fail_count += 1
            if maxfail and fail_count >= maxfail:
                break

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

    final_failed = fail_count + (1 if ratchet_errors else 0)
    summary = {
        "passed": pass_count,
        "failed": fail_count,
        "skipped": skip_count,
        "total": len(rows),
        "duration_ms": total_duration_ms,
    }

    if ctx.output_format == "json" or ns.json:
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "check-run",
            "run_id": ctx.run_id,
            "status": "ok" if final_failed == 0 else "error",
            "summary": summary,
            "slow_threshold_ms": slow_threshold_ms,
            "slow_checks": slow_rows,
            "ratchet_errors": ratchet_errors,
            "rows": rows,
        }
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

    if ns.json_report:
        report_path = ensure_evidence_path(ctx, Path(ns.json_report))
        report_payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "check-run-report",
            "run_id": ctx.run_id,
            "summary": summary,
            "slow_threshold_ms": slow_threshold_ms,
            "slow_checks": slow_rows,
            "ratchet_errors": ratchet_errors,
            "rows": rows,
        }
        report_path.write_text(json.dumps(report_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    slow_report = getattr(ns, "slow_report", None)
    if slow_report:
        slow_path = ensure_evidence_path(ctx, Path(slow_report))
        slow_path.write_text(
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
        profile_path.write_text(
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


def run_check_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    show_source_id = str(getattr(ns, "show_source", "") or "").strip()
    if show_source_id:
        path = _check_source_path(ctx, show_source_id)
        if path is None:
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": f"unknown check id: {show_source_id}"}, sort_keys=True) if ctx.output_format == "json" or getattr(ns, "json", False) else f"unknown check id: {show_source_id}")
            return ERR_USER
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "id": show_source_id, "source": path}
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or getattr(ns, "json", False) else path)
        return 0
    sub = ns.check_cmd
    if not sub and bool(getattr(ns, "list_checks", False)):
        sub = "list"
    if sub == "run":
        try:
            return _run_check_registry(ctx, ns)
        except Exception as exc:  # pragma: no cover
            print(f"internal check runner error: {exc}")
            return ERR_CONTRACT
    if sub == "list":
        checks = list_checks()
        if not (ctx.output_format == "json" or ns.json):
            for check in checks:
                print(check.check_id)
            return 0
        payload = {
            "schema_name": CHECK_LIST,
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "checks": [
                {
                    "id": check.check_id,
                    "title": check.title,
                    "domain": check.domain,
                    "description": check.description,
                    "severity": check.severity.value,
                    "category": check.category.value,
                    "fix_hint": check.fix_hint,
                    "slow": check.slow,
                    "tags": list(check_tags(check)),
                    "effects": list(check.effects),
                    "owners": list(check.owners),
                    "external_tools": list(check.external_tools),
                }
                for check in checks
            ],
        }
        validate_self(CHECK_LIST, payload)
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "explain":
        check = get_check(ns.check_id)
        if check is None:
            print(f"unknown check id: {ns.check_id}")
            return ERR_USER
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "id": check.check_id,
            "title": check.title,
            "domain": check.domain,
            "description": check.description,
            "severity": check.severity.value,
            "category": check.category.value,
            "tags": list(check_tags(check)),
            "effects": list(check.effects),
            "owners": list(check.owners),
            "failure_modes": ["policy violation", "contract drift", "hygiene drift"],
            "how_to_fix": check.fix_hint,
            "source": _check_source_path(ctx, check.check_id),
        }
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "runtime-contracts":
        payload = runtime_contracts_payload(ctx.repo_root)
        if ns.out_file:
            ensure_evidence_path(ctx, Path(ns.out_file)).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0 if payload["status"] == "pass" else ERR_USER

    if sub == "all":
        return _run_domain(ctx, "all", fail_fast=ns.fail_fast, label="all")
    if sub == "domain":
        return _run_domain(ctx, ns.domain, fail_fast=ns.fail_fast)
    if sub == "license":
        return _run_domain(ctx, "license", fail_fast=ns.fail_fast, label="license")
    if sub in {"make", "docs", "configs"}:
        suite = {"make": "makefiles", "docs": "docs", "configs": "configs"}[sub]
        return _run_suite_domain(ctx, suite, sub, ns.fail_fast)

    if sub == "repo":
        if getattr(ns, "repo_check", "all") == "module-size":
            return _run_native_check(ctx, check_module_size, "oversized atlasctl modules detected:", "module size policy passed")
        return _run_domain(ctx, "repo")
    if sub == "layout":
        return _run_native_check(ctx, check_layout_contract, "layout contract failed:", "layout contract passed", limit=200)
    if sub == "shell":
        checks = [check for cid in SHELL_POLICY_CHECK_IDS if (check := get_check(cid)) is not None]
        failed, results = run_function_checks(ctx.repo_root, checks)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok" if failed == 0 else "error",
            "group": "shell",
            "failed_count": failed,
            "total_count": len(results),
            "results": [
                {
                    "id": row.id,
                    "status": row.status,
                    "errors": row.errors,
                    "warnings": row.warnings,
                }
                for row in results
            ],
        }
        if ctx.output_format == "json" or ns.json:
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"check shell: {payload['status']} ({failed}/{len(results)} failed)")
            for row in payload["results"]:
                if row["status"] != "pass":
                    print(f"- {row['id']}: {', '.join(row['errors'][:2])}")
        return 0 if failed == 0 else ERR_USER
    if sub == "obs":
        return _run(ctx, ["python3", "packages/atlasctl/src/atlasctl/observability/contracts/metrics/check_metrics_contract.py"])
    if sub == "stack-report":
        return _run(ctx, ["python3", "packages/atlasctl/src/atlasctl/stack/validate_stack_report.py"])
    if sub == "forbidden-paths":
        refs_code, ref_errors = check_make_scripts_references(ctx.repo_root)
        path_code, path_errors = check_make_forbidden_paths(ctx.repo_root)
        if ref_errors:
            _print_errors("make scripts reference policy failed:", ref_errors, "", limit=200)
        if path_errors:
            _print_errors("forbidden make recipe paths detected:", path_errors, "")
        if refs_code == 0 and path_code == 0:
            print("make forbidden path checks passed")
            return 0
        return ERR_USER
    if sub == "python-runtime-artifacts":
        code, errors = check_python_runtime_artifacts(ctx.repo_root, fix=bool(getattr(ns, "fix", False)))
        if errors:
            if code == 0:
                for err in errors:
                    print(err)
            else:
                _print_errors("python runtime artifact policy failed:", errors, "", limit=200)
        else:
            print("python runtime artifact policy passed")
        return code
    if sub == "generate-scripts-sbom":
        code, outputs = generate_scripts_sbom(ctx.repo_root, ns.lock, ns.out)
        for out in outputs:
            print(out)
        return code

    check_map: dict[str, tuple[NativeCheck, str, str, int | None, str]] = {
        "cli-help": (check_script_help, "script help contract failed:", "script help contract passed", None, "- "),
        "ownership": (check_script_ownership, "script ownership coverage failed:", "script ownership coverage passed", None, "- "),
        "root-bin-shims": (check_root_bin_shims, "root bin shim policy failed:", "root bin shim policy passed", None, "- "),
        "duplicate-script-names": (check_duplicate_script_names, "duplicate dash/underscore script names detected:", "no duplicate script names", None, "- "),
        "bin-entrypoints": (check_bin_entrypoints, "bin entrypoint cap failed:", "scripts/bin cap ok", None, "- "),
        "make-scripts-refs": (check_make_scripts_references, "make scripts reference policy failed:", "make scripts reference policy passed", 200, "- "),
        "docs-scripts-refs": (check_docs_scripts_references, "docs scripts reference policy failed:", "docs scripts reference policy passed", 200, "- "),
        "make-help": (check_make_help, "", "make help output is deterministic", None, ""),
        "no-xtask": (check_no_xtask_refs, "xtask references detected:", "no xtask references detected", None, "- "),
        "no-python-shebang-outside-packages": (check_no_executable_python_outside_packages, "forbidden executable python files detected:", "no executable python files outside packages", None, "- "),
        "forbidden-top-dirs": (check_forbidden_top_dirs, "forbidden top-level directories detected:", "no forbidden top-level directories", None, "- "),
        "module-size": (check_module_size, "oversized atlasctl modules detected:", "module size policy passed", None, "- "),
        "ops-generated-tracked": (check_ops_generated_tracked, "tracked files detected under ops/_generated:", "ops/_generated has no tracked files", None, "- "),
        "tracked-timestamps": (check_tracked_timestamp_paths, "tracked timestamp-like paths detected:", "no tracked timestamp-like paths detected", None, "- "),
        "committed-generated-hygiene": (check_committed_generated_hygiene, "committed generated hygiene violations detected:", "committed generated directories contain deterministic assets only", None, "- "),
        "effects-lint": (check_effects_lint, "effects lint failed:", "effects lint passed", None, "- "),
        "naming-intent-lint": (check_naming_intent_lint, "naming intent lint failed:", "naming intent lint passed", None, "- "),
        "make-command-allowlist": (check_make_command_allowlist, "make command allowlist check failed:", "make command allowlist check passed", 200, "- "),
        "python-migration-exceptions-expiry": (check_python_migration_exceptions_expiry, "python migration exceptions have expired:", "python migration exceptions expiry check passed", None, "- "),
        "python-lock": (check_python_lock, "invalid scripts requirements lock entries:", "scripts python lock format passed", None, "- "),
        "scripts-lock-sync": (check_scripts_lock_sync, "scripts lock drift detected:", "scripts lock check passed", None, "- "),
        "no-adhoc-python": (check_no_adhoc_python, "no ad-hoc python script check failed", "no ad-hoc python script check passed", 200, "- unregistered python file outside tools package: "),
        "no-direct-python-invocations": (check_no_direct_python_invocations, "direct python invocation policy check failed:", "direct python invocation policy check passed", 200, "- "),
        "no-direct-bash-invocations": (check_no_direct_bash_invocations, "direct bash invocation policy check failed:", "direct bash invocation policy check passed", 200, "- "),
        "invocation-parity": (check_invocation_parity, "invocation parity check failed:", "invocation parity check passed", None, "- "),
        "scripts-surface-docs-drift": (check_scripts_surface_docs_drift, "scripts command surface docs drift detected:", "scripts command surface docs drift check passed", None, "- "),
        "script-errors": (check_script_errors, "structured error contract failed:", "structured error contract passed", None, "- "),
        "script-write-roots": (check_script_write_roots, "script write-root policy failed:", "script write-root policy passed", None, "- "),
        "script-tool-guards": (check_script_tool_guards, "scripts using kubectl/helm/kind/k6 without version guard:", "script tool guard check passed", None, "- "),
        "script-shim-expiry": (check_script_shim_expiry, "script shim expiry check failed", "script shim expiry check passed", None, "- "),
        "script-shims-minimal": (check_script_shims_minimal, "script shim minimality check failed:", "script shim minimality check passed", None, "- "),
        "venv-location-policy": (check_venv_location_policy, "venv location policy failed:", "venv location policy passed", 200, "- forbidden .venv location: "),
        "repo-script-boundaries": (check_repo_script_boundaries, "repo script boundary check failed:", "repo script boundary check passed", 200, "- "),
        "atlas-cli-contract": (check_atlas_scripts_cli_contract, "atlasctl cli contract check failed:", "atlasctl cli contract check passed", None, "- "),
        "bijux-boundaries": (check_atlasctl_boundaries, "atlasctl boundary check failed", "atlasctl boundary check passed", None, "- "),
    }
    if sub in check_map:
        fn, title, ok_message, limit, prefix = check_map[sub]
        return _run_native_check(ctx, fn, title, ok_message, limit=limit, prefix=prefix)

    return ERR_USER


def configure_check_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("check", help="area-based checks mapped from scripts/areas")
    parser.add_argument("--fail-fast", action="store_true", help="stop after first failing check in multi-check runs")
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    parser.add_argument("--list", dest="list_checks", action="store_true", help="list registered checks")
    parser.add_argument("--show-source", help="print source file for check id")
    parser_sub = parser.add_subparsers(dest="check_cmd", required=False)

    parser_sub.add_parser("all", help="run all native atlasctl checks")
    run = parser_sub.add_parser("run", help="run registered checks with pytest-like output")
    run.add_argument("--quiet", dest="run_quiet", action="store_true", help="one line per check: PASS/FAIL/SKIP")
    run.add_argument("--info", dest="run_info", action="store_true", help="default info output mode with id + timing")
    run.add_argument("--verbose", dest="run_verbose", action="store_true", help="include timing, owners, and failure hints")
    run.add_argument("--maxfail", type=int, default=0, help="stop after N failing checks (0 disables)")
    run.add_argument("--failfast", action="store_true", help="stop after first failing check")
    run.add_argument("--fail-fast", dest="failfast", action="store_true", help="stop after first failing check")
    run.add_argument("--durations", type=int, default=0, help="show N slowest checks in summary")
    run.add_argument("--junitxml", help="write junit xml output path")
    run.add_argument("--junit-xml", dest="junit_xml", help="write junit xml output path")
    run.add_argument("--json-report", help="write json report output path")
    run.add_argument("--jsonl", action="store_true", help="stream JSONL row events and summary")
    run.add_argument("--slow-report", help="write slow checks report output path")
    run.add_argument("--slow-threshold-ms", type=int, default=800, help="threshold for slow checks report")
    run.add_argument("--slow-ratchet-config", default="configs/policy/slow-checks-ratchet.json", help="slow-check ratchet config json")
    run.add_argument("--profile", action="store_true", help="emit check run performance profile artifact")
    run.add_argument("--profile-out", help="performance profile output path")
    run.add_argument("--match", help="glob pattern over check ids/titles")
    run.add_argument("--group", help="filter checks by group/domain")
    run.add_argument("--require-markers", action="append", default=[], help="require check markers/tags (repeatable or comma-separated)")
    run.add_argument("--select", help="check selector, e.g. atlasctl::docs::check_x")
    run.add_argument("check_target", nargs="?", help="fully-qualified check id, e.g. atlasctl::docs::check_x")
    run.add_argument("--json", action="store_true", help="emit JSON output")
    parser_sub.add_parser("list", help="list registered checks")
    explain = parser_sub.add_parser("explain", help="explain a check id")
    explain.add_argument("check_id")
    runtime = parser_sub.add_parser("runtime-contracts", help="run unified runtime contract checks and emit artifact")
    runtime.add_argument("--out-file", help="optional artifact output path under evidence root")
    domain = parser_sub.add_parser("domain", help="run checks for one domain")
    domain.add_argument("domain", choices=check_domains())

    parser_sub.add_parser("layout", help="run layout checks")
    parser_sub.add_parser("shell", help="run shell policy checks")
    parser_sub.add_parser("make", help="run makefile checks")
    parser_sub.add_parser("docs", help="run docs checks")
    parser_sub.add_parser("configs", help="run configs checks")
    parser_sub.add_parser("license", help="run licensing checks")
    repo = parser_sub.add_parser("repo", help="run repo hygiene checks")
    repo.add_argument("repo_check", nargs="?", choices=["all", "module-size"], default="all")
    parser_sub.add_parser("obs", help="run observability checks")
    parser_sub.add_parser("stack-report", help="validate stack report contracts")

    for name, help_text in [
        ("cli-help", "validate script/CLI help coverage"),
        ("ownership", "validate script ownership coverage"),
        ("bin-entrypoints", "validate scripts/bin entrypoint cap"),
        ("root-bin-shims", "validate root bin shim minimalism policy"),
        ("duplicate-script-names", "validate duplicate script names"),
        ("make-scripts-refs", "validate no makefile references to scripts paths"),
        ("docs-scripts-refs", "validate docs contain no scripts/ path references"),
        ("make-help", "validate deterministic make help output"),
        ("forbidden-paths", "forbid scripts/xtask/tools direct recipe paths"),
        ("no-xtask", "forbid xtask references outside ADR history"),
        ("no-python-shebang-outside-packages", "forbid executable python scripts outside packages/"),
        ("forbidden-top-dirs", "fail if forbidden top-level directories exist"),
        ("module-size", "enforce max python module LOC budget"),
        ("ops-generated-tracked", "fail if ops/_generated contains tracked files"),
        ("tracked-timestamps", "fail if tracked paths contain timestamp-like directories"),
        ("committed-generated-hygiene", "fail on runtime/timestamped artifacts in committed generated directories"),
        ("effects-lint", "forbid runtime effects leakage in pure/query HTTP layers"),
        ("naming-intent-lint", "forbid generic helpers naming in crates tree"),
        ("make-command-allowlist", "enforce direct make recipe command allowlist"),
        ("python-migration-exceptions-expiry", "fail on expired python migration exceptions"),
        ("python-lock", "validate scripts python lockfile line format"),
        ("scripts-lock-sync", "validate scripts lockfile remains in sync with pyproject dev deps"),
        ("no-adhoc-python", "validate no unregistered ad-hoc python scripts are tracked"),
        ("no-direct-python-invocations", "forbid direct python invocations in docs/makefiles"),
        ("no-direct-bash-invocations", "forbid direct bash script invocations in docs/makefiles"),
        ("invocation-parity", "validate atlasctl invocation parity in make/docs"),
        ("scripts-surface-docs-drift", "validate scripts surface docs coverage from python tooling config"),
        ("script-errors", "validate structured script error contract"),
        ("script-write-roots", "validate scripts write only under approved roots"),
        ("script-tool-guards", "validate tool-using scripts include guard calls"),
        ("script-shim-expiry", "validate shim expiry metadata and budget"),
        ("script-shims-minimal", "validate shim wrappers remain minimal and deterministic"),
        ("venv-location-policy", "validate .venv locations are restricted"),
        ("repo-script-boundaries", "validate script location boundaries and transition exceptions"),
        ("atlas-cli-contract", "validate atlasctl CLI help/version deterministic contract"),
        ("bijux-boundaries", "validate atlasctl import boundaries"),
    ]:
        parser_sub.add_parser(name, help=help_text)

    runtime_artifacts = parser_sub.add_parser("python-runtime-artifacts", help="validate runtime python artifacts stay outside tracked paths")
    runtime_artifacts.add_argument("--fix", action="store_true", help="remove forbidden runtime artifact paths in-place")
    sbom = parser_sub.add_parser("generate-scripts-sbom", help="emit python lock SBOM json")
    sbom.add_argument("--lock", required=True)
    sbom.add_argument("--out", required=True)
