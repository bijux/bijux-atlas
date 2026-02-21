from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from ...checks.runner import domains as check_domains
from ...checks.runner import run_domain
from ...checks.registry import get_check, list_checks
from ...checks.repo.command_contracts import runtime_contracts_payload
from ...core.context import RunContext
from ...core.fs import ensure_evidence_path
from ...lint.runner import run_suite
from ...check.native import (
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
from ...checks.repo.module_size import check_module_size


def _run(ctx: RunContext, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False)
    return proc.returncode


def run_check_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = ns.check_cmd
    if sub == "list":
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "checks": [
                {
                    "id": c.check_id,
                    "domain": c.domain,
                    "description": c.description,
                    "severity": c.severity.value,
                    "category": c.category.value,
                    "fix_hint": c.fix_hint,
                    "slow": c.slow,
                    "external_tools": list(c.external_tools),
                }
                for c in list_checks()
            ],
        }
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
            "failure_modes": ["policy violation", "contract drift", "hygiene drift"],
            "how_to_fix": check.fix_hint,
        }
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if sub == "runtime-contracts":
        payload = runtime_contracts_payload(ctx.repo_root)
        if ns.out_file:
            out = ensure_evidence_path(ctx, Path(ns.out_file))
            out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps(payload, sort_keys=True) if ctx.output_format == "json" or ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0 if payload["status"] == "pass" else 1
    if sub == "all":
        code, payload = run_domain(ctx.repo_root, "all", fail_fast=ns.fail_fast)
        if ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"check all: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
        return code
    if sub == "domain":
        code, payload = run_domain(ctx.repo_root, ns.domain, fail_fast=ns.fail_fast)
        if ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"check {ns.domain}: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
        return code
    if sub == "license":
        code, payload = run_domain(ctx.repo_root, "license", fail_fast=ns.fail_fast)
        if ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"check license: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
        return code
    if sub in {"make", "docs", "configs"}:
        suite_name = {"make": "makefiles", "docs": "docs", "configs": "configs"}[sub]
        code, payload = run_suite(ctx.repo_root, suite_name, fail_fast=ns.fail_fast)
        if ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"check {sub}: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
        return code
    if sub == "repo":
        if getattr(ns, "repo_check", "all") == "module-size":
            code, errors = check_module_size(ctx.repo_root)
            if errors:
                print("oversized atlasctl modules detected:")
                for err in errors:
                    print(f"- {err}")
            else:
                print("module size policy passed")
            return code
        code, payload = run_domain(ctx.repo_root, "repo")
        if ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"check repo: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
        return code
    if sub == "layout":
        code, errors = check_layout_contract(ctx.repo_root)
        if errors:
            print("layout contract failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("layout contract passed")
        return code
    if sub == "obs":
        return _run(
            ctx,
            [
                "python3",
                "packages/atlasctl/src/atlasctl/obs/contracts/check_metrics_contract.py",
            ],
        )
    if sub == "stack-report":
        return _run(
            ctx,
            ["python3", "packages/atlasctl/src/atlasctl/stack/validate_stack_report.py"],
        )
    if sub == "cli-help":
        code, errors = check_script_help(ctx.repo_root)
        if errors:
            print("script help contract failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("script help contract passed")
        return code
    if sub == "ownership":
        code, errors = check_script_ownership(ctx.repo_root)
        if errors:
            print("script ownership coverage failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("script ownership coverage passed")
        return code
    if sub == "root-bin-shims":
        code, errors = check_root_bin_shims(ctx.repo_root)
        if errors:
            print("root bin shim policy failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("root bin shim policy passed")
        return code
    if sub == "duplicate-script-names":
        code, errors = check_duplicate_script_names(ctx.repo_root)
        if errors:
            print("duplicate dash/underscore script names detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no duplicate script names")
        return code
    if sub == "bin-entrypoints":
        code, errors = check_bin_entrypoints(ctx.repo_root)
        if errors:
            print("bin entrypoint cap failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("scripts/bin cap ok")
        return code
    if sub == "make-scripts-refs":
        code, errors = check_make_scripts_references(ctx.repo_root)
        if errors:
            print("make scripts reference policy failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("make scripts reference policy passed")
        return code
    if sub == "docs-scripts-refs":
        code, errors = check_docs_scripts_references(ctx.repo_root)
        if errors:
            print("docs scripts reference policy failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("docs scripts reference policy passed")
        return code
    if sub == "make-help":
        code, errors = check_make_help(ctx.repo_root)
        if errors:
            for err in errors:
                print(err)
        else:
            print("make help output is deterministic")
        return code
    if sub == "forbidden-paths":
        code_script_refs, script_ref_errors = check_make_scripts_references(ctx.repo_root)
        code_paths, errors = check_make_forbidden_paths(ctx.repo_root)
        if script_ref_errors:
            print("make scripts reference policy failed:")
            for err in script_ref_errors[:200]:
                print(f"- {err}")
        if errors:
            print("forbidden make recipe paths detected:")
            for err in errors:
                print(f"- {err}")
        if code_script_refs == 0 and code_paths == 0:
            print("make forbidden path checks passed")
            return 0
        return 1
    if sub == "no-xtask":
        code, errors = check_no_xtask_refs(ctx.repo_root)
        if errors:
            print("xtask references detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no xtask references detected")
        return code
    if sub == "no-python-shebang-outside-packages":
        code, errors = check_no_executable_python_outside_packages(ctx.repo_root)
        if errors:
            print("forbidden executable python files detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no executable python files outside packages")
        return code
    if sub == "forbidden-top-dirs":
        code, errors = check_forbidden_top_dirs(ctx.repo_root)
        if errors:
            print("forbidden top-level directories detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no forbidden top-level directories")
        return code
    if sub == "module-size":
        code, errors = check_module_size(ctx.repo_root)
        if errors:
            print("oversized atlasctl modules detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("module size policy passed")
        return code
    if sub == "ops-generated-tracked":
        code, errors = check_ops_generated_tracked(ctx.repo_root)
        if errors:
            print("tracked files detected under ops/_generated:")
            for err in errors:
                print(f"- {err}")
        else:
            print("ops/_generated has no tracked files")
        return code
    if sub == "tracked-timestamps":
        code, errors = check_tracked_timestamp_paths(ctx.repo_root)
        if errors:
            print("tracked timestamp-like paths detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no tracked timestamp-like paths detected")
        return code
    if sub == "committed-generated-hygiene":
        code, errors = check_committed_generated_hygiene(ctx.repo_root)
        if errors:
            print("committed generated hygiene violations detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("committed generated directories contain deterministic assets only")
        return code
    if sub == "effects-lint":
        code, errors = check_effects_lint(ctx.repo_root)
        if errors:
            print("effects lint failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("effects lint passed")
        return code
    if sub == "naming-intent-lint":
        code, errors = check_naming_intent_lint(ctx.repo_root)
        if errors:
            print("naming intent lint failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("naming intent lint passed")
        return code
    if sub == "make-command-allowlist":
        code, errors = check_make_command_allowlist(ctx.repo_root)
        if errors:
            print("make command allowlist check failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("make command allowlist check passed")
        return code
    if sub == "python-migration-exceptions-expiry":
        code, errors = check_python_migration_exceptions_expiry(ctx.repo_root)
        if errors:
            print("python migration exceptions have expired:")
            for err in errors:
                print(f"- {err}")
        else:
            print("python migration exceptions expiry check passed")
        return code
    if sub == "python-lock":
        code, errors = check_python_lock(ctx.repo_root)
        if errors:
            print("invalid scripts requirements lock entries:")
            for err in errors:
                print(f"- {err}")
        else:
            print("scripts python lock format passed")
        return code
    if sub == "scripts-lock-sync":
        code, errors = check_scripts_lock_sync(ctx.repo_root)
        if errors:
            print("scripts lock drift detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("scripts lock check passed")
        return code
    if sub == "no-adhoc-python":
        code, errors = check_no_adhoc_python(ctx.repo_root)
        if errors:
            print("no ad-hoc python script check failed")
            for err in errors[:200]:
                print(f"- unregistered python file outside tools package: {err}")
        else:
            print("no ad-hoc python script check passed")
        return code
    if sub == "no-direct-python-invocations":
        code, errors = check_no_direct_python_invocations(ctx.repo_root)
        if errors:
            print("direct python invocation policy check failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("direct python invocation policy check passed")
        return code
    if sub == "no-direct-bash-invocations":
        code, errors = check_no_direct_bash_invocations(ctx.repo_root)
        if errors:
            print("direct bash invocation policy check failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("direct bash invocation policy check passed")
        return code
    if sub == "invocation-parity":
        code, errors = check_invocation_parity(ctx.repo_root)
        if errors:
            print("invocation parity check failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("invocation parity check passed")
        return code
    if sub == "scripts-surface-docs-drift":
        code, errors = check_scripts_surface_docs_drift(ctx.repo_root)
        if errors:
            print("scripts command surface docs drift detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("scripts command surface docs drift check passed")
        return code
    if sub == "script-errors":
        code, errors = check_script_errors(ctx.repo_root)
        if errors:
            print("structured error contract failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("structured error contract passed")
        return code
    if sub == "script-write-roots":
        code, errors = check_script_write_roots(ctx.repo_root)
        if errors:
            print("script write-root policy failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("script write-root policy passed")
        return code
    if sub == "script-tool-guards":
        code, errors = check_script_tool_guards(ctx.repo_root)
        if errors:
            print("scripts using kubectl/helm/kind/k6 without version guard:")
            for err in errors:
                print(f"- {err}")
        else:
            print("script tool guard check passed")
        return code
    if sub == "script-shim-expiry":
        code, errors = check_script_shim_expiry(ctx.repo_root)
        if errors:
            print("script shim expiry check failed")
            for err in errors:
                print(f"- {err}")
        else:
            print("script shim expiry check passed")
        return code
    if sub == "script-shims-minimal":
        code, errors = check_script_shims_minimal(ctx.repo_root)
        if errors:
            print("script shim minimality check failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("script shim minimality check passed")
        return code
    if sub == "venv-location-policy":
        code, errors = check_venv_location_policy(ctx.repo_root)
        if errors:
            print("venv location policy failed:")
            for err in errors[:200]:
                print(f"- forbidden .venv location: {err}")
        else:
            print("venv location policy passed")
        return code
    if sub == "python-runtime-artifacts":
        code, errors = check_python_runtime_artifacts(ctx.repo_root, fix=bool(getattr(ns, "fix", False)))
        if errors:
            if code == 0:
                for err in errors:
                    print(err)
            else:
                print("python runtime artifact policy failed:")
                for err in errors[:200]:
                    print(f"- {err}")
        else:
            print("python runtime artifact policy passed")
        return code
    if sub == "repo-script-boundaries":
        code, errors = check_repo_script_boundaries(ctx.repo_root)
        if errors:
            print("repo script boundary check failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("repo script boundary check passed")
        return code
    if sub == "atlas-cli-contract":
        code, errors = check_atlas_scripts_cli_contract(ctx.repo_root)
        if errors:
            print("atlasctl cli contract check failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("atlasctl cli contract check passed")
        return code
    if sub == "bijux-boundaries":
        code, errors = check_atlasctl_boundaries(ctx.repo_root)
        if errors:
            print("atlasctl boundary check failed")
            for err in errors:
                print(f"- {err}")
        else:
            print("atlasctl boundary check passed")
        return code
    if sub == "generate-scripts-sbom":
        code, outputs = generate_scripts_sbom(ctx.repo_root, ns.lock, ns.out)
        for out in outputs:
            print(out)
        return code
    return 2


def configure_check_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("check", help="area-based checks mapped from scripts/areas")
    p.add_argument("--fail-fast", action="store_true", help="stop after first failing check in multi-check runs")
    p.add_argument("--json", action="store_true", help="emit JSON output")
    p_sub = p.add_subparsers(dest="check_cmd", required=True)
    p_sub.add_parser("all", help="run all native atlasctl checks")
    p_sub.add_parser("list", help="list registered checks")
    explain = p_sub.add_parser("explain", help="explain a check id")
    explain.add_argument("check_id")
    runtime = p_sub.add_parser("runtime-contracts", help="run unified runtime contract checks and emit artifact")
    runtime.add_argument("--out-file", help="optional artifact output path under evidence root")
    domain = p_sub.add_parser("domain", help="run checks for one domain")
    domain.add_argument("domain", choices=check_domains())
    p_sub.add_parser("layout", help="run layout checks")
    p_sub.add_parser("make", help="run makefile checks")
    p_sub.add_parser("docs", help="run docs checks")
    p_sub.add_parser("configs", help="run configs checks")
    p_sub.add_parser("license", help="run licensing checks")
    repo = p_sub.add_parser("repo", help="run repo hygiene checks (forbidden roots, refs, caches/artifacts)")
    repo.add_argument("repo_check", nargs="?", choices=["all", "module-size"], default="all")
    p_sub.add_parser("obs", help="run observability checks")
    p_sub.add_parser("stack-report", help="validate stack report contracts")
    p_sub.add_parser("cli-help", help="validate script/CLI help coverage")
    p_sub.add_parser("ownership", help="validate script ownership coverage")
    p_sub.add_parser("bin-entrypoints", help="validate scripts/bin entrypoint cap")
    p_sub.add_parser("root-bin-shims", help="validate root bin shim minimalism policy")
    p_sub.add_parser("duplicate-script-names", help="validate duplicate script names")
    p_sub.add_parser("make-scripts-refs", help="validate no makefile references to scripts paths")
    p_sub.add_parser("docs-scripts-refs", help="validate docs contain no scripts/ path references")
    p_sub.add_parser("make-help", help="validate deterministic make help output")
    p_sub.add_parser("forbidden-paths", help="forbid scripts/xtask/tools direct recipe paths")
    p_sub.add_parser("no-xtask", help="forbid xtask references outside ADR history")
    p_sub.add_parser(
        "no-python-shebang-outside-packages",
        help="forbid executable python scripts outside packages/",
    )
    p_sub.add_parser("forbidden-top-dirs", help="fail if forbidden top-level directories exist")
    p_sub.add_parser("module-size", help="enforce max python module LOC budget")
    p_sub.add_parser("ops-generated-tracked", help="fail if ops/_generated contains tracked files")
    p_sub.add_parser("tracked-timestamps", help="fail if tracked paths contain timestamp-like directories")
    p_sub.add_parser(
        "committed-generated-hygiene",
        help="fail on runtime/timestamped artifacts in committed generated directories",
    )
    p_sub.add_parser("effects-lint", help="forbid runtime effects leakage in pure/query HTTP layers")
    p_sub.add_parser("naming-intent-lint", help="forbid generic helpers naming in crates tree")
    p_sub.add_parser("make-command-allowlist", help="enforce direct make recipe command allowlist")
    p_sub.add_parser("python-migration-exceptions-expiry", help="fail on expired python migration exceptions")
    p_sub.add_parser("python-lock", help="validate scripts python lockfile line format")
    p_sub.add_parser("scripts-lock-sync", help="validate scripts lockfile remains in sync with pyproject dev deps")
    p_sub.add_parser("no-adhoc-python", help="validate no unregistered ad-hoc python scripts are tracked")
    p_sub.add_parser("no-direct-python-invocations", help="forbid direct python invocations in docs/makefiles")
    p_sub.add_parser("no-direct-bash-invocations", help="forbid direct bash scripts invocations in docs/makefiles")
    p_sub.add_parser("invocation-parity", help="validate atlasctl invocation parity in make/docs")
    p_sub.add_parser("scripts-surface-docs-drift", help="validate scripts surface docs coverage from python tooling config")
    p_sub.add_parser("script-errors", help="validate structured script error contract")
    p_sub.add_parser("script-write-roots", help="validate scripts write only under approved roots")
    p_sub.add_parser("script-tool-guards", help="validate tool-using scripts include guard calls")
    p_sub.add_parser("script-shim-expiry", help="validate shim expiry metadata and budget")
    p_sub.add_parser("script-shims-minimal", help="validate shim wrappers remain minimal and deterministic")
    p_sub.add_parser("venv-location-policy", help="validate .venv locations are restricted")
    runtime = p_sub.add_parser("python-runtime-artifacts", help="validate runtime python artifacts stay outside tracked paths")
    runtime.add_argument("--fix", action="store_true", help="remove forbidden runtime artifact paths in-place")
    p_sub.add_parser("repo-script-boundaries", help="validate script location boundaries and transition exceptions")
    p_sub.add_parser("atlas-cli-contract", help="validate atlasctl CLI help/version deterministic contract")
    p_sub.add_parser("bijux-boundaries", help="validate atlasctl import boundaries")
    sbom = p_sub.add_parser("generate-scripts-sbom", help="emit python lock SBOM json")
    sbom.add_argument("--lock", required=True)
    sbom.add_argument("--out", required=True)
