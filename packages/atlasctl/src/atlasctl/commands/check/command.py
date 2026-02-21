from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Callable

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
from ...lint.runner import run_suite

NativeCheck = Callable[[Path], tuple[int, list[str]]]
SHELL_POLICY_CHECK_IDS: tuple[str, ...] = (
    "repo.shell_location_policy",
    "repo.shell_strict_mode",
    "repo.shell_no_direct_python",
    "repo.shell_no_network_fetch",
    "repo.shell_invocation_boundary",
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
    code, payload = run_suite(ctx.repo_root, suite_name, fail_fast=fail_fast)
    if ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"check {label}: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
    return code


def run_check_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = ns.check_cmd
    if sub == "list":
        payload = {
            "schema_name": CHECK_LIST,
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "checks": [
                {
                    "id": check.check_id,
                    "domain": check.domain,
                    "description": check.description,
                    "severity": check.severity.value,
                    "category": check.category.value,
                    "fix_hint": check.fix_hint,
                    "slow": check.slow,
                    "tags": list(check_tags(check)),
                    "external_tools": list(check.external_tools),
                }
                for check in list_checks()
            ],
        }
        validate_self(CHECK_LIST, payload)
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "explain":
        check = get_check(ns.check_id)
        if check is None:
            print(f"unknown check id: {ns.check_id}")
            return 2
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "id": check.check_id,
            "domain": check.domain,
            "description": check.description,
            "severity": check.severity.value,
            "category": check.category.value,
            "tags": list(check_tags(check)),
            "failure_modes": ["policy violation", "contract drift", "hygiene drift"],
            "how_to_fix": check.fix_hint,
        }
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "runtime-contracts":
        payload = runtime_contracts_payload(ctx.repo_root)
        if ns.out_file:
            ensure_evidence_path(ctx, Path(ns.out_file)).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0 if payload["status"] == "pass" else 1

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
        return 0 if failed == 0 else 1
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
        return 1
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

    return 2


def configure_check_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("check", help="area-based checks mapped from scripts/areas")
    parser.add_argument("--fail-fast", action="store_true", help="stop after first failing check in multi-check runs")
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    parser_sub = parser.add_subparsers(dest="check_cmd", required=True)

    parser_sub.add_parser("all", help="run all native atlasctl checks")
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
