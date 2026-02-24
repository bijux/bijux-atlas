"""Check command runtime dispatch."""

from __future__ import annotations

import argparse
import inspect
import json
import re

from ...checks.registry import alias_expiry_violations, check_rename_aliases, check_tags, check_tree, get_check, list_checks, marker_vocabulary, resolve_aliases
from ...checks.report import (
    build_failures_payload,
    build_triage_failures_payload,
    build_triage_slow_payload,
    report_from_payload,
    resolve_last_run_report,
)
from ...checks.effects import CheckEffect, normalize_effect
from ...checks.registry import generate_registry_json
from ...contracts.ids import CHECK_LIST, CHECK_TAXONOMY
from ...contracts.validate_self import validate_self
from ...core.exit_codes import ERR_CONTRACT, ERR_USER
from ...core.fs import ensure_evidence_path
from ...core.meta.owners import load_owner_catalog


def _run_check_failures(ctx, ns: argparse.Namespace) -> int:
    try:
        report_path = resolve_last_run_report(str(ns.last_run))
    except FileNotFoundError as exc:
        print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "error", "error": str(exc)}, sort_keys=True) if (ctx.output_format == "json" or ns.json) else str(exc))
        return ERR_USER
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    group = str(getattr(ns, "group", "") or "").strip()
    report = report_from_payload(payload)
    out = build_failures_payload(source=report_path.as_posix(), report=report, group=group)
    if ctx.output_format == "json" or ns.json:
        print(json.dumps(out, sort_keys=True))
        return 0 if not out["failures"] else ERR_USER
    if not out["failures"]:
        print(f"failures: none ({group or 'all'})")
        return 0
    print(f"failures ({group or 'all'}): {len(out['failures'])}")
    for row in out["failures"]:
        print(f"- {row['id']}: {row['detail'] or row['hint']}")
    return ERR_USER


def _run_check_triage_slow(ctx, ns: argparse.Namespace) -> int:
    report_path = resolve_last_run_report(str(ns.last_run))
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    report = report_from_payload(payload)
    out = build_triage_slow_payload(source=report_path.as_posix(), report=report, top=int(getattr(ns, "top", 10) or 10))
    if ctx.output_format == "json" or ns.json:
        print(json.dumps(out, sort_keys=True))
    else:
        for row in out["rows"]:
            print(f"{row['id']}\t{row['domain']}\t{row['duration_ms']}ms")
    return 0


def _run_check_triage_failures(ctx, ns: argparse.Namespace) -> int:
    report_path = resolve_last_run_report(str(ns.last_run))
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    report = report_from_payload(payload)
    out = build_triage_failures_payload(source=report_path.as_posix(), report=report)
    if ctx.output_format == "json" or ns.json:
        print(json.dumps(out, sort_keys=True))
        return 0 if not out["failed_count"] else ERR_USER
    if not out["failed_count"]:
        print("triage failures: none")
        return 0
    print(f"triage failures: {out['failed_count']}")
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
        aliases = resolve_aliases()
        expiry = aliases[0].expires_on.isoformat() if aliases else None
        violations = alias_expiry_violations()
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "kind": "check-rename-report",
            "alias_expires_on": expiry,
            "alias_expiry_violations": violations,
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
        checks = list(list_checks())
        domain_filter = str(getattr(ns, "domain_filter", "") or "").strip()
        category_filter = str(getattr(ns, "category", "") or "").strip()
        if domain_filter:
            checks = [check for check in checks if check.domain == domain_filter]
        if category_filter:
            checks = [check for check in checks if check.category.value == category_filter]
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
                    "canonical_id": str(getattr(check, "canonical_id", check.check_id)),
                    "gate": next((tag.removeprefix("gate:") for tag in check.tags if tag.startswith("gate:")), check.domain),
                    "title": check.title,
                    "domain": check.domain,
                    "description": check.description,
                    "intent": check.intent or check.description,
                    "severity": check.severity.value,
                    "category": check.category.value,
                    "result_code": check.result_code,
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
        tree = check_tree()
        if ctx.output_format == "json" or ns.json:
            payload = {
                "schema_name": CHECK_TAXONOMY,
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "tree": [
                    {"domain": domain, "areas": [{"name": area, "checks": ids} for area, ids in sorted(areas.items())]}
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
    if sub == "gates":
        checks = list_checks()
        gates: dict[str, list[str]] = {}
        for check in checks:
            gate = next((tag.removeprefix("gate:") for tag in check.tags if tag.startswith("gate:")), check.domain)
            gates.setdefault(gate, []).append(check.check_id)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "kind": "check-gates",
            "gates": [{"gate": gate, "checks": sorted(ids), "count": len(ids)} for gate, ids in sorted(gates.items())],
        }
        if ctx.output_format == "json" or ns.json:
            print(json.dumps(payload, sort_keys=True))
            return 0
        for row in payload["gates"]:
            print(f"{row['gate']} ({row['count']})")
            for check_id in row["checks"]:
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
        for marker in marker_vocabulary():
            grouped.setdefault(marker, [])
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
        if "." in str(ns.check_id) and not bool(getattr(ns, "legacy_id", False)) and not (ctx.output_format == "json" or ns.json):
            print(f"warning: using legacy dotted check id without --legacy-id: {ns.check_id}")
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
            "gate": next((tag.removeprefix("gate:") for tag in check.tags if tag.startswith("gate:")), check.domain),
        }
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "doc":
        if "." in str(ns.check_id) and not bool(getattr(ns, "legacy_id", False)) and not (ctx.output_format == "json" or ns.json):
            print(f"warning: using legacy dotted check id without --legacy-id: {ns.check_id}")
        check = get_check(ns.check_id)
        if check is None:
            print(f"unknown check id: {ns.check_id}")
            return ERR_USER
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "check-doc",
            "status": "ok",
            "id": check.check_id,
            "title": check.title,
            "domain": check.domain,
            "category": check.category.value,
            "intent": check.intent or check.description,
            "owners": list(check.owners),
            "result_code": check.result_code,
            "remediation": {
                "short": check.remediation_short or check.fix_hint,
                "link": check.remediation_link,
            },
        }
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "doctor":
        checks = list_checks()
        canonical_pattern = re.compile(r"^checks_[a-z0-9]+_[a-z0-9]+_[a-z0-9_]+$")
        canonical_errors: list[str] = []
        owner_errors: list[str] = []
        effect_errors: list[str] = []
        valid_effects = {
            CheckEffect.FS_READ.value,
            CheckEffect.FS_WRITE.value,
            CheckEffect.SUBPROCESS.value,
            CheckEffect.GIT.value,
            CheckEffect.NETWORK.value,
        }
        owner_catalog = set(load_owner_catalog(ctx.repo_root).owners)
        seen: set[str] = set()
        duplicate_errors: list[str] = []
        for check in checks:
            cid = str(check.check_id)
            if cid in seen:
                duplicate_errors.append(cid)
            seen.add(cid)
            if canonical_pattern.match(cid) is None:
                canonical_errors.append(cid)
            else:
                parts = cid.split("_", 3)
                if len(parts) >= 3 and str(check.domain) != parts[1]:
                    canonical_errors.append(f"{cid}: domain segment `{parts[1]}` != `{check.domain}`")
            if not tuple(check.owners):
                owner_errors.append(f"{cid}: missing owner")
            else:
                for owner in tuple(check.owners):
                    if str(owner) not in owner_catalog:
                        owner_errors.append(f"{cid}: unknown owner `{owner}`")
            for effect in tuple(check.effects):
                normalized = normalize_effect(str(effect))
                if normalized not in valid_effects:
                    effect_errors.append(f"{cid}: unknown effect `{effect}`")
        drift_errors: list[str] = []
        try:
            _out, changed = generate_registry_json(ctx.repo_root, check_only=True)
            if changed:
                drift_errors.append("registry drift detected: run `./bin/atlasctl gen checks-registry`")
        except Exception as exc:
            drift_errors.append(f"registry generation failed: {exc}")
        alias_errors = alias_expiry_violations()
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "checks-doctor",
            "status": "ok" if not (canonical_errors or duplicate_errors or owner_errors or effect_errors or drift_errors or alias_errors) else "error",
            "summary": {
                "total": len(checks),
                "canonical_errors": len(canonical_errors),
                "duplicate_errors": len(duplicate_errors),
                "owner_errors": len(owner_errors),
                "effect_errors": len(effect_errors),
                "drift_errors": len(drift_errors),
                "alias_errors": len(alias_errors),
            },
            "errors": {
                "canonical": canonical_errors,
                "duplicates": sorted(set(duplicate_errors)),
                "owners": sorted(set(owner_errors)),
                "effects": sorted(set(effect_errors)),
                "drift": drift_errors,
                "aliases": alias_errors,
            },
        }
        if ctx.output_format == "json" or ns.json:
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"checks={payload['summary']['total']}")
            print("checks doctor: pass" if payload["status"] == "ok" else "checks doctor: fail")
            if payload["status"] != "ok":
                for key, values in payload["errors"].items():
                    if values:
                        print(f"- {key}:")
                        for value in values:
                            print(f"  - {value}")
        return 0 if payload["status"] == "ok" else ERR_USER
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
        domain = {"make": "make", "docs": "docs", "configs": "configs"}[sub]
        return impl._run_domain(ctx, domain, fail_fast=ns.fail_fast, label=sub)

    if sub == "repo":
        if getattr(ns, "repo_check", "all") == "module-size":
            return impl._run_native_check(ctx, impl.check_module_size, "oversized atlasctl modules detected:", "module size policy passed")
        if getattr(ns, "repo_check", "all") == "hygiene":
            from ...repo_hygiene import run_repo_hygiene_checks

            result = run_repo_hygiene_checks(ctx.repo_root)
            if ctx.output_format == "json" or ns.json:
                print(json.dumps({"schema_version": 1, "tool": "atlasctl", "kind": "repo-hygiene", "status": result.status, "checks": result.checks}, sort_keys=True))
            else:
                print("repo hygiene: pass" if result.status == "ok" else "repo hygiene: fail")
                if result.status != "ok":
                    for name, rows in result.checks.items():
                        if rows:
                            print(f"- {name}:")
                            for row in rows[:50]:
                                print(f"  - {row}")
            return 0 if result.status == "ok" else ERR_USER
        return impl._run_domain(ctx, "repo")
    if sub == "repo-hygiene":
        from ...repo_hygiene import run_repo_hygiene_checks

        result = run_repo_hygiene_checks(ctx.repo_root)
        if ctx.output_format == "json" or ns.json:
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "kind": "repo-hygiene", "status": result.status, "checks": result.checks}, sort_keys=True))
        else:
            print("repo hygiene: pass" if result.status == "ok" else "repo hygiene: fail")
            if result.status != "ok":
                for name, rows in result.checks.items():
                    if rows:
                        print(f"- {name}:")
                        for row in rows[:50]:
                            print(f"  - {row}")
        return 0 if result.status == "ok" else ERR_USER
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
            ["python3", "packages/atlasctl/src/atlasctl/checks/domains/policies/check_make_targets_drift.py"],
        )
    if sub == "make-delegation-only":
        return impl._run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/makefiles/policies/check_make_delegation_only.py"],
        )
    if sub == "workflow-calls-atlasctl":
        return impl._run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/domains/ops/contracts/check_workflow_calls_atlasctl.py"],
        )
    if sub == "ci-surface-documented":
        from ...checks.tools.docs_ci_surface_documented import main as run_ci_surface_documented

        return run_ci_surface_documented()
    if sub == "ops-mk-contract":
        return impl._run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/checks/layout/makefiles/policies/check_ops_mk_contract.py"],
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
        "no-xtask": (
            impl.check_no_xtask_refs,
            "legacy task-runner references detected:",
            "no legacy task-runner references detected",
            None,
            "- ",
        ),
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
