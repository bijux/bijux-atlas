"""Check command runtime dispatch."""

from __future__ import annotations

import argparse
import inspect
import json
from pathlib import Path

from ...checks.registry import check_rename_aliases, check_tags, get_check, list_checks
from ...contracts.ids import CHECK_LIST, CHECK_TAXONOMY
from ...contracts.validate_self import validate_self
from ...core.exit_codes import ERR_CONTRACT, ERR_USER
from ...core.fs import ensure_evidence_path


def _resolve_failures_report(last_run: str) -> Path:
    path = Path(last_run)
    if path.is_file():
        return path
    if not path.exists():
        raise FileNotFoundError(f"last-run path does not exist: {last_run}")
    candidates = sorted(path.rglob("*.json"), key=lambda p: p.stat().st_mtime, reverse=True)
    for candidate in candidates:
        try:
            payload = json.loads(candidate.read_text(encoding="utf-8"))
        except Exception:
            continue
        if payload.get("kind") in {"check-run", "check-run-report"} and isinstance(payload.get("rows"), list):
            return candidate
    raise FileNotFoundError(f"no check-run report json found under: {last_run}")


def _run_check_failures(ctx, ns: argparse.Namespace) -> int:
    try:
        report_path = _resolve_failures_report(str(ns.last_run))
    except FileNotFoundError as exc:
        print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": str(exc)}, sort_keys=True) if (ctx.output_format == "json" or ns.json) else str(exc))
        return ERR_USER
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    group = str(getattr(ns, "group", "") or "").strip()
    rows = payload.get("rows", [])
    if group:
        rows = [row for row in rows if str(row.get("id", "")).startswith(f"checks_{group}_")]
    failed = [row for row in rows if row.get("status") == "FAIL"]
    out = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "check-failures",
        "status": "ok",
        "source": report_path.as_posix(),
        "group": group or "all",
        "failed_count": len(failed),
        "failures": [
            {
                "id": str(row.get("id", "")),
                "domain": str(row.get("domain", "")),
                "hint": str(row.get("hint", "")),
                "detail": str(row.get("detail", "")),
            }
            for row in failed
        ],
    }
    if ctx.output_format == "json" or ns.json:
        print(json.dumps(out, sort_keys=True))
        return 0 if not failed else ERR_USER
    if not failed:
        print(f"failures: none ({group or 'all'})")
        return 0
    print(f"failures ({group or 'all'}): {len(failed)}")
    for row in out["failures"]:
        print(f"- {row['id']}: {row['detail'] or row['hint']}")
    return ERR_USER


def _run_check_triage_slow(ctx, ns: argparse.Namespace) -> int:
    report_path = _resolve_failures_report(str(ns.last_run))
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    rows = payload.get("rows", [])
    ranked = sorted(
        [row for row in rows if row.get("status") in {"PASS", "FAIL"}],
        key=lambda row: int(row.get("duration_ms", 0)),
        reverse=True,
    )[: max(1, int(getattr(ns, "top", 10) or 10))]
    out = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "check-triage-slow",
        "status": "ok",
        "source": report_path.as_posix(),
        "rows": [{"id": row.get("id"), "domain": row.get("domain"), "duration_ms": int(row.get("duration_ms", 0))} for row in ranked],
    }
    if ctx.output_format == "json" or ns.json:
        print(json.dumps(out, sort_keys=True))
    else:
        for row in out["rows"]:
            print(f"{row['id']}\t{row['domain']}\t{row['duration_ms']}ms")
    return 0


def _run_check_triage_failures(ctx, ns: argparse.Namespace) -> int:
    report_path = _resolve_failures_report(str(ns.last_run))
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    rows = [row for row in payload.get("rows", []) if row.get("status") == "FAIL"]
    grouped: dict[str, dict[str, int]] = {}
    for row in rows:
        domain = str(row.get("domain", "unknown"))
        rid = str(row.get("id", ""))
        area = rid.split("_")[2] if rid.startswith("checks_") and len(rid.split("_")) > 2 else "general"
        bucket = grouped.setdefault(domain, {})
        bucket[area] = int(bucket.get(area, 0)) + 1
    out = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "check-triage-failures",
        "status": "ok",
        "source": report_path.as_posix(),
        "failed_count": len(rows),
        "groups": {domain: {area: count for area, count in sorted(areas.items())} for domain, areas in sorted(grouped.items())},
    }
    if ctx.output_format == "json" or ns.json:
        print(json.dumps(out, sort_keys=True))
        return 0 if not rows else ERR_USER
    if not rows:
        print("triage failures: none")
        return 0
    print(f"triage failures: {len(rows)}")
    for domain, areas in out["groups"].items():
        area_bits = ", ".join(f"{area}={count}" for area, count in areas.items())
        print(f"- {domain}: {area_bits}")
    return ERR_USER


def run(ctx, ns: argparse.Namespace) -> int:
    # Import lazily to avoid import cycles; command.py hosts shared helper implementations.
    from . import command as impl

    show_source_id = str(getattr(ns, "show_source", "") or "").strip()
    if show_source_id:
        path = impl._check_source_path(ctx, show_source_id)
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
            return impl._run_check_registry(ctx, ns)
        except Exception as exc:  # pragma: no cover
            print(f"internal check runner error: {exc}")
            return ERR_CONTRACT
    if sub == "rename-report":
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "kind": "check-rename-report",
            "renames": [{"old": old, "new": new} for old, new in check_rename_aliases().items()],
        }
        print(json.dumps(payload, sort_keys=True) if (ns.json or ctx.output_format == "json") else "\n".join(f"{row['old']} -> {row['new']}" for row in payload["renames"]))
        return 0
    if sub == "failures":
        return _run_check_failures(ctx, ns)
    if sub == "triage-slow":
        return _run_check_triage_slow(ctx, ns)
    if sub == "triage-failures":
        return _run_check_triage_failures(ctx, ns)
    if sub == "list":
        checks = list_checks()
        if not (ctx.output_format == "json" or ns.json):
            for check in checks:
                print(check.check_id)
            return 0
        domains: dict[str, dict[str, object]] = {}
        for check in checks:
            segments = check.check_id.split("_")
            area = segments[2] if len(segments) > 3 else "general"
            bucket = domains.setdefault(
                check.domain,
                {"count": 0, "areas": {}, "checks": []},
            )
            bucket["count"] = int(bucket["count"]) + 1
            areas = bucket["areas"]
            areas[area] = int(areas.get(area, 0)) + 1
            bucket["checks"].append(check.check_id)
        use_taxonomy = str(getattr(ns, "cmd", "")) == "checks"
        payload = {
            "schema_name": CHECK_TAXONOMY if use_taxonomy else CHECK_LIST,
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "taxonomy": [{"domain": domain, **meta} for domain, meta in sorted(domains.items())],
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
        validate_self(CHECK_TAXONOMY if use_taxonomy else CHECK_LIST, payload)
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "tree":
        checks = list_checks()
        tree: dict[str, dict[str, list[str]]] = {}
        for check in checks:
            parts = check.check_id.split("_")
            domain = parts[1] if len(parts) > 1 else check.domain
            area = parts[2] if len(parts) > 2 else "general"
            tree.setdefault(domain, {}).setdefault(area, []).append(check.check_id)
        if ctx.output_format == "json" or ns.json:
            payload = {
                "schema_name": CHECK_TAXONOMY,
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "tree": [
                    {"domain": domain, "areas": [{"name": area, "checks": sorted(ids)} for area, ids in sorted(areas.items())]}
                    for domain, areas in sorted(tree.items())
                ],
            }
            print(json.dumps(payload, sort_keys=True))
            return 0
        for domain, areas in sorted(tree.items()):
            print(domain)
            for area, ids in sorted(areas.items()):
                print(f"  {area}")
                for check_id in sorted(ids):
                    print(f"    - {check_id}")
        return 0
    if sub == "owners":
        checks = list_checks()
        ownership: dict[str, list[str]] = {}
        for check in checks:
            for owner in check.owners or ("unowned",):
                ownership.setdefault(owner, []).append(check.check_id)
        if ctx.output_format == "json" or ns.json:
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "kind": "check-owners",
                "owners": [{"owner": owner, "checks": sorted(ids), "count": len(ids)} for owner, ids in sorted(ownership.items())],
            }
            print(json.dumps(payload, sort_keys=True))
            return 0
        for owner, ids in sorted(ownership.items()):
            print(f"{owner} ({len(ids)})")
            for check_id in sorted(ids):
                print(f"- {check_id}")
        return 0
    if sub == "groups":
        checks = list_checks()
        grouped: dict[str, list[str]] = {}
        for check in checks:
            for group in check_tags(check):
                grouped.setdefault(group, []).append(check.check_id)
            grouped.setdefault(f"{check.domain}-slow", [])
            grouped.setdefault(f"{check.domain}-fast", [])
            if check.slow:
                grouped[f"{check.domain}-slow"].append(check.check_id)
            else:
                grouped[f"{check.domain}-fast"].append(check.check_id)
        if ctx.output_format == "json" or ns.json:
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "kind": "check-groups",
                "groups": [{"group": group, "checks": sorted(ids), "count": len(ids)} for group, ids in sorted(grouped.items())],
            }
            print(json.dumps(payload, sort_keys=True))
            return 0
        for group, ids in sorted(grouped.items()):
            print(f"{group} ({len(ids)})")
            for check_id in sorted(ids):
                print(f"- {check_id}")
        return 0
    if sub == "slow":
        checks = [check for check in list_checks() if check.slow]
        if ctx.output_format == "json" or ns.json:
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "kind": "check-slow",
                "checks": [check.check_id for check in checks],
                "count": len(checks),
            }
            print(json.dumps(payload, sort_keys=True))
            return 0
        for check in checks:
            print(check.check_id)
        return 0
    if sub == "explain":
        check = get_check(ns.check_id)
        if check is None:
            print(f"unknown check id: {ns.check_id}")
            return ERR_USER
        module_doc = ""
        try:
            module = inspect.getmodule(check.fn)
            module_doc = str(getattr(module, "__doc__", "") or "").strip()
        except Exception:
            module_doc = ""
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
            "source": impl._check_source_path(ctx, check.check_id),
            "module": check.fn.__module__,
            "callable": check.fn.__name__,
            "module_docstring": module_doc,
        }
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "runtime-contracts":
        payload = impl.runtime_contracts_payload(ctx.repo_root)
        if ns.out_file:
            ensure_evidence_path(ctx, ns.out_file).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0 if payload["status"] == "pass" else ERR_USER

    if sub == "all":
        return impl._run_domain(ctx, "all", fail_fast=ns.fail_fast, label="all")
    if sub == "domain":
        return impl._run_domain(ctx, ns.domain, fail_fast=ns.fail_fast)
    if sub == "license":
        return impl._run_domain(ctx, "license", fail_fast=ns.fail_fast, label="license")
    if sub in {"make", "docs", "configs"}:
        suite = {"make": "makefiles", "docs": "docs", "configs": "configs"}[sub]
        return impl._run_suite_domain(ctx, suite, sub, ns.fail_fast)

    if sub == "repo":
        if getattr(ns, "repo_check", "all") == "module-size":
            return impl._run_native_check(ctx, impl.check_module_size, "oversized atlasctl modules detected:", "module size policy passed")
        return impl._run_domain(ctx, "repo")
    if sub == "layout":
        return impl._run_native_check(ctx, impl.check_layout_contract, "layout contract failed:", "layout contract passed", limit=200)
    if sub == "shell":
        checks = [check for cid in impl.SHELL_POLICY_CHECK_IDS if (check := get_check(cid)) is not None]
        failed, results = impl.run_function_checks(ctx.repo_root, checks)
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
        return impl._run(ctx, ["python3", "packages/atlasctl/src/atlasctl/observability/contracts/metrics/check_metrics_contract.py"])
    if sub == "stack-report":
        return impl._run(ctx, ["python3", "packages/atlasctl/src/atlasctl/stack/validate_stack_report.py"])
    if sub == "forbidden-paths":
        refs_code, ref_errors = impl.check_make_scripts_references(ctx.repo_root)
        path_code, path_errors = impl.check_make_forbidden_paths(ctx.repo_root)
        if ref_errors:
            impl._print_errors("make scripts reference policy failed:", ref_errors, "", limit=200)
        if path_errors:
            impl._print_errors("forbidden make recipe paths detected:", path_errors, "")
        if refs_code == 0 and path_code == 0:
            print("make forbidden path checks passed")
            return 0
        return ERR_USER
    if sub == "python-runtime-artifacts":
        code, errors = impl.check_python_runtime_artifacts(ctx.repo_root, fix=bool(getattr(ns, "fix", False)))
        if errors:
            if code == 0:
                for err in errors:
                    print(err)
            else:
                impl._print_errors("python runtime artifact policy failed:", errors, "", limit=200)
        else:
            print("python runtime artifact policy passed")
        return code
    if sub == "generate-scripts-sbom":
        code, outputs = impl.generate_scripts_sbom(ctx.repo_root, ns.lock, ns.out)
        for out in outputs:
            print(out)
        return code
    if sub == "make-targets-drift":
        return impl._run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/domains/policies/make/impl/check_make_targets_drift.py"],
        )
    if sub == "make-delegation-only":
        return impl._run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/makefiles/policies/check_make_delegation_only.py"],
        )
    if sub == "workflow-calls-atlasctl":
        return impl._run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/workflows/check_workflow_calls_atlasctl.py"],
        )
    if sub == "ci-surface-documented":
        from atlasctl.checks.layout.docs.check_ci_surface_documented import main as check_ci_surface_documented_main

        return check_ci_surface_documented_main()
    if sub == "ops-mk-contract":
        return impl._run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/makefiles/policies/ci/check_ops_mk_contract.py"],
        )
    if sub == "checks-registry-drift":
        try:
            _out, changed = impl.generate_registry_json(ctx.repo_root, check_only=True)
        except Exception as exc:
            print(f"checks registry validation failed: {exc}")
            return ERR_CONTRACT
        if changed:
            print("checks registry drift detected: run `./bin/atlasctl gen checks-registry`")
            return ERR_USER
        print("checks registry drift check passed")
        return 0

    check_map: dict[str, tuple[impl.NativeCheck, str, str, int | None, str]] = {
        "cli-help": (impl.check_script_help, "script help contract failed:", "script help contract passed", None, "- "),
        "ownership": (impl.check_script_ownership, "script ownership coverage failed:", "script ownership coverage passed", None, "- "),
        "root-bin-shims": (impl.check_root_bin_shims, "root bin shim policy failed:", "root bin shim policy passed", None, "- "),
        "duplicate-script-names": (impl.check_duplicate_script_names, "duplicate dash/underscore script names detected:", "no duplicate script names", None, "- "),
        "bin-entrypoints": (impl.check_bin_entrypoints, "bin entrypoint cap failed:", "scripts/bin cap ok", None, "- "),
        "make-scripts-refs": (impl.check_make_scripts_references, "make scripts reference policy failed:", "make scripts reference policy passed", 200, "- "),
        "docs-scripts-refs": (impl.check_docs_scripts_references, "docs scripts reference policy failed:", "docs scripts reference policy passed", 200, "- "),
        "make-help": (impl.check_make_help, "", "make help output is deterministic", None, ""),
        "no-xtask": (impl.check_no_xtask_refs, "xtask references detected:", "no xtask references detected", None, "- "),
        "no-python-shebang-outside-packages": (impl.check_no_executable_python_outside_packages, "forbidden executable python files detected:", "no executable python files outside packages", None, "- "),
        "forbidden-top-dirs": (impl.check_forbidden_top_dirs, "forbidden top-level directories detected:", "no forbidden top-level directories", None, "- "),
        "module-size": (impl.check_module_size, "oversized atlasctl modules detected:", "module size policy passed", None, "- "),
        "ops-generated-tracked": (impl.check_ops_generated_tracked, "tracked files detected under ops/_generated:", "ops/_generated has no tracked files", None, "- "),
        "tracked-timestamps": (impl.check_tracked_timestamp_paths, "tracked timestamp-like paths detected:", "no tracked timestamp-like paths detected", None, "- "),
        "committed-generated-hygiene": (impl.check_committed_generated_hygiene, "committed generated hygiene violations detected:", "committed generated directories contain deterministic assets only", None, "- "),
        "effects-lint": (impl.check_effects_lint, "effects lint failed:", "effects lint passed", None, "- "),
        "naming-intent-lint": (impl.check_naming_intent_lint, "naming intent lint failed:", "naming intent lint passed", None, "- "),
        "make-command-allowlist": (impl.check_make_command_allowlist, "make command allowlist check failed:", "make command allowlist check passed", 200, "- "),
        "python-migration-exceptions-expiry": (impl.check_python_migration_exceptions_expiry, "python migration exceptions have expired:", "python migration exceptions expiry check passed", None, "- "),
        "python-lock": (impl.check_python_lock, "invalid scripts requirements lock entries:", "scripts python lock format passed", None, "- "),
        "scripts-lock-sync": (impl.check_scripts_lock_sync, "scripts lock drift detected:", "scripts lock check passed", None, "- "),
        "no-adhoc-python": (impl.check_no_adhoc_python, "no ad-hoc python script check failed", "no ad-hoc python script check passed", 200, "- unregistered python file outside tools package: "),
        "no-direct-python-invocations": (impl.check_no_direct_python_invocations, "direct python invocation policy check failed:", "direct python invocation policy check passed", 200, "- "),
        "no-direct-bash-invocations": (impl.check_no_direct_bash_invocations, "direct bash invocation policy check failed:", "direct bash invocation policy check passed", 200, "- "),
        "invocation-parity": (impl.check_invocation_parity, "invocation parity check failed:", "invocation parity check passed", None, "- "),
        "scripts-surface-docs-drift": (impl.check_scripts_surface_docs_drift, "scripts command surface docs drift detected:", "scripts command surface docs drift check passed", None, "- "),
        "script-errors": (impl.check_script_errors, "structured error contract failed:", "structured error contract passed", None, "- "),
        "script-write-roots": (impl.check_script_write_roots, "script write-root policy failed:", "script write-root policy passed", None, "- "),
        "script-tool-guards": (impl.check_script_tool_guards, "scripts using kubectl/helm/kind/k6 without version guard:", "script tool guard check passed", None, "- "),
        "script-shim-expiry": (impl.check_script_shim_expiry, "script shim expiry check failed", "script shim expiry check passed", None, "- "),
        "script-shims-minimal": (impl.check_script_shims_minimal, "script shim minimality check failed:", "script shim minimality check passed", None, "- "),
        "venv-location-policy": (impl.check_venv_location_policy, "venv location policy failed:", "venv location policy passed", 200, "- forbidden .venv location: "),
        "repo-script-boundaries": (impl.check_repo_script_boundaries, "repo script boundary check failed:", "repo script boundary check passed", 200, "- "),
        "atlas-cli-contract": (impl.check_atlas_scripts_cli_contract, "atlasctl cli contract check failed:", "atlasctl cli contract check passed", None, "- "),
        "bijux-boundaries": (impl.check_atlasctl_boundaries, "atlasctl boundary check failed", "atlasctl boundary check passed", None, "- "),
    }
    if sub in check_map:
        fn, title, ok_message, limit, prefix = check_map[sub]
        return impl._run_native_check(ctx, fn, title, ok_message, limit=limit, prefix=prefix)

    return ERR_USER


__all__ = ["run"]
