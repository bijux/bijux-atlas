from __future__ import annotations

import argparse
import json
import subprocess
from dataclasses import dataclass
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Callable

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path


@dataclass(frozen=True)
class OpsCheck:
    check_id: str
    description: str
    cmd: list[str]
    actionable: str


def _check(check_id: str, description: str, cmd: list[str], actionable: str) -> OpsCheck:
    return OpsCheck(check_id, description, cmd, actionable)


LINT_CHECKS: list[OpsCheck] = [
    _check(
        "ops-run-entrypoints",
        "Ensure ops run entrypoints policy",
        ["python3", "scripts/areas/layout/check_ops_run_entrypoints.py"],
        "Route public ops entrypoints through ops/run/* wrappers.",
    ),
    _check(
        "ops-shell-policy",
        "Ensure ops shell policy",
        ["python3", "scripts/areas/layout/check_ops_shell_policy.py"],
        "Use shared ops shell wrappers and policy-compliant shell structure.",
    ),
    _check(
        "ops-evidence-writes",
        "Ensure ops evidence write policy",
        ["python3", "scripts/areas/layout/check_no_ops_evidence_writes.py"],
        "Write runtime artifacts only under artifacts/evidence or approved allowlist roots.",
    ),
    _check(
        "ops-load-suite-manifest",
        "Validate load suite manifest",
        [
            "./scripts/bin/bijux-atlas-scripts",
            "run",
            "./tools/bijux-atlas-scripts/src/bijux_atlas_scripts/load/validate_suite_manifest.py",
        ],
        "Fix suite schema violations and keep suite manifest aligned with contract.",
    ),
    _check(
        "ops-k8s-surface-generate",
        "Generate k8s test surface",
        ["python3", "scripts/areas/ops/generate_k8s_test_surface.py"],
        "Regenerate k8s test surface metadata from manifest inputs.",
    ),
    _check(
        "ops-k8s-suites-validate",
        "Validate k8s suite declarations",
        ["python3", "ops/k8s/tests/validate_suites.py"],
        "Align k8s suite groups with manifest declarations.",
    ),
    _check(
        "ops-k8s-test-lib",
        "Validate k8s test library contract",
        ["python3", "scripts/areas/ops/check_k8s_test_lib.py"],
        "Keep k8s test helper library canonical and deduplicated.",
    ),
    _check(
        "ops-k8s-checks-layout",
        "Validate k8s checks layout budget",
        ["python3", "scripts/areas/ops/check_k8s_checks_layout.py"],
        "Split oversized checks directories and keep grouping contract.",
    ),
    _check(
        "ops-orphan-contracts",
        "Validate ops contract references",
        ["python3", "ops/_lint/no-orphan-contract.py"],
        "Register missing contracts and update ops/INDEX.md links.",
    ),
    _check(
        "ops-orphan-suites",
        "Validate ops suite references",
        ["python3", "ops/_lint/no-orphan-suite.py"],
        "Keep suite manifests aligned with scenario/test files.",
    ),
    _check(
        "ops-unowned-file",
        "Validate ops ownership coverage",
        ["python3", "ops/_lint/no-unowned-file.py"],
        "Add missing ownership entries under ops/_meta/ownership.json.",
    ),
    _check(
        "ops-json-schema-coverage",
        "Validate ops schema coverage",
        ["python3", "ops/_lint/json-schema-coverage.py"],
        "Ensure all ops contracts have schema coverage or explicit exclusion.",
    ),
    _check(
        "ops-no-unpinned-images",
        "Validate image pinning contract",
        ["python3", "ops/_lint/no-unpinned-images.py"],
        "Pin image references by digest and update pin policy exceptions when needed.",
    ),
    _check(
        "ops-profile-goldens",
        "Validate observability profile goldens",
        [
            "./scripts/bin/bijux-atlas-scripts",
            "run",
            "./tools/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/check_profile_goldens.py",
        ],
        "Refresh approved profile goldens through the documented update flow.",
    ),
    _check(
        "ops-tool-versions",
        "Validate pinned tool versions",
        ["python3", "scripts/areas/layout/check_tool_versions.py", "kind", "kubectl", "helm", "k6"],
        "Update tool pins and local toolchain to match configs/ops/tool-versions.json.",
    ),
    _check(
        "ops-env-validate",
        "Validate ops environment contract",
        ["make", "-s", "ops-env-validate"],
        "Fix env schema violations and keep configs/ops/env.schema.json authoritative.",
    ),
    _check(
        "ops-layout-lint",
        "Validate ops layout contracts",
        ["make", "-s", "ops-layout-lint"],
        "Resolve ops layout drift and keep INDEX/contracts synchronized.",
    ),
]


def _run_check(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output.strip()


def _load_relaxations(repo_root: Path) -> tuple[dict[str, dict[str, str]], list[str]]:
    path = repo_root / "configs/policy/ops-lint-relaxations.json"
    if not path.exists():
        return {}, []
    payload = json.loads(path.read_text(encoding="utf-8"))
    entries = payload.get("relaxations", [])
    today = date.today()
    indexed: dict[str, dict[str, str]] = {}
    errors: list[str] = []
    for item in entries:
        if not isinstance(item, dict):
            continue
        check_id = str(item.get("check_id", "")).strip()
        expires_on = str(item.get("expires_on", "")).strip()
        if not check_id:
            errors.append("ops-lint relaxation missing check_id")
            continue
        if not expires_on:
            errors.append(f"ops-lint relaxation `{check_id}` missing expires_on")
            continue
        try:
            expiry = date.fromisoformat(expires_on)
        except ValueError:
            errors.append(f"ops-lint relaxation `{check_id}` has invalid expires_on `{expires_on}`")
            continue
        if expiry < today:
            errors.append(f"ops-lint relaxation `{check_id}` expired on {expires_on}")
            continue
        indexed[check_id] = {
            "owner": str(item.get("owner", "")),
            "issue": str(item.get("issue", "")),
            "expires_on": expires_on,
            "reason": str(item.get("reason", "")),
        }
    return indexed, errors


def _ops_report(run_id: str, checks: list[dict[str, object]], started_at: str, ended_at: str) -> dict[str, object]:
    failed = [c for c in checks if c.get("status") == "fail"]
    relaxed = [c for c in checks if c.get("status") == "relaxed"]
    return {
        "schema_version": 1,
        "tool": "bijux-atlas-scripts",
        "run_id": run_id,
        "status": "fail" if failed else "pass",
        "started_at": started_at,
        "ended_at": ended_at,
        "checks": checks,
        "failed_count": len(failed),
        "relaxed_count": len(relaxed),
        "total_count": len(checks),
    }


def _validate_report_schema(repo_root: Path, payload: dict[str, object]) -> None:
    import jsonschema

    schema_path = repo_root / "configs/contracts/ops-lint-output.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)


def _run_checks(
    ctx: RunContext,
    checks: list[OpsCheck],
    fail_fast: bool,
    report_format: str,
    emit_artifacts: bool,
    runner: Callable[[list[str], Path], tuple[int, str]] = _run_check,
) -> int:
    repo_root = ctx.repo_root
    relaxations, relaxation_errors = _load_relaxations(repo_root)
    started_at = datetime.now(timezone.utc).isoformat()
    rows: list[dict[str, object]] = []

    if relaxation_errors:
        for err in relaxation_errors:
            rows.append(
                {
                    "id": "ops-lint-relaxations",
                    "description": "Validate ops lint relaxations",
                    "status": "fail",
                    "command": "configs/policy/ops-lint-relaxations.json",
                    "actionable": "Fix malformed or expired relaxations.",
                    "error": err,
                }
            )
        fail_fast = True

    for check in checks:
        code, output = runner(check.cmd, repo_root)
        status = "pass" if code == 0 else "fail"
        row: dict[str, object] = {
            "id": check.check_id,
            "description": check.description,
            "status": status,
            "command": " ".join(check.cmd),
            "actionable": check.actionable,
        }
        if status == "fail":
            row["error"] = output
            relaxation = relaxations.get(check.check_id)
            if relaxation:
                row["status"] = "relaxed"
                row["relaxation"] = relaxation
        rows.append(row)
        if fail_fast and row["status"] == "fail":
            break

    ended_at = datetime.now(timezone.utc).isoformat()
    payload = _ops_report(ctx.run_id, rows, started_at, ended_at)
    _validate_report_schema(repo_root, payload)

    if emit_artifacts:
        out_path = ensure_evidence_path(
            ctx,
            ctx.evidence_root / "ops" / "lint" / ctx.run_id / "report.json",
        )
        out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            "ops lint: "
            f"status={payload['status']} "
            f"checks={payload['total_count']} "
            f"failed={payload['failed_count']} "
            f"relaxed={payload['relaxed_count']}"
        )
        for row in rows:
            if row["status"] in {"fail", "relaxed"}:
                first_line = (
                    str(row.get("error", "")).splitlines()[:1][0]
                    if row.get("error")
                    else "check failed"
                )
                print(f"- {row['status'].upper()} {row['id']}: {first_line}")
                print(f"  fix: {row['actionable']}")

    return 0 if payload["status"] == "pass" else 1


def _run_simple_cmd(ctx: RunContext, cmd: list[str], report_format: str) -> int:
    code, output = _run_check(cmd, ctx.repo_root)
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas-scripts",
        "run_id": ctx.run_id,
        "status": "pass" if code == 0 else "fail",
        "command": " ".join(cmd),
        "output": output,
    }
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(output)
    return code


def run_ops_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.ops_cmd == "check":
        steps = [
            ["./scripts/bin/bijux-atlas-scripts", "ops", "lint", "--report", ns.report, "--emit-artifacts"],
            ["./scripts/bin/bijux-atlas-scripts", "ops", "contracts-check", "--report", ns.report],
            ["./scripts/bin/bijux-atlas-scripts", "ops", "suites-check", "--report", ns.report],
            ["./scripts/bin/bijux-atlas-scripts", "ops", "schema-check", "--report", ns.report],
            ["env", "CACHE_STATUS_STRICT=0", "make", "-s", "ops-cache-status"],
            ["make", "-s", "pins/check"],
            ["./scripts/bin/bijux-atlas-scripts", "ops", "surface", "--report", ns.report],
            ["python3", "scripts/areas/layout/check_ops_index_surface.py"],
        ]
        for cmd in steps:
            code, output = _run_check(cmd, ctx.repo_root)
            if output:
                print(output)
            if code != 0:
                return code
        return 0

    if ns.ops_cmd == "lint":
        if ns.fix:
            for cmd in (
                ["python3", "scripts/areas/layout/generate_ops_surface_meta.py"],
                ["python3", "scripts/areas/docs/generate_ops_contracts_doc.py"],
            ):
                code, output = _run_check(cmd, ctx.repo_root)
                if code != 0:
                    if output:
                        print(output)
                    return code
        return _run_checks(
            ctx,
            checks=LINT_CHECKS,
            fail_fast=ns.fail_fast,
            report_format=ns.report,
            emit_artifacts=ns.emit_artifacts,
        )

    if ns.ops_cmd == "surface":
        if ns.fix:
            return _run_simple_cmd(ctx, ["python3", "scripts/areas/layout/generate_ops_surface_meta.py"], ns.report)
        return _run_simple_cmd(ctx, ["python3", "scripts/areas/layout/check_ops_surface_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-check":
        return _run_simple_cmd(ctx, ["python3", "scripts/areas/layout/validate_ops_contracts.py"], ns.report)

    if ns.ops_cmd == "suites-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-orphan-suite.py"],
            ns.report,
        )

    if ns.ops_cmd == "schema-check":
        return _run_simple_cmd(ctx, ["python3", "scripts/areas/layout/validate_ops_contracts.py"], ns.report)

    if ns.ops_cmd == "tool-versions-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-floating-tool-versions.py"],
            ns.report,
        )

    if ns.ops_cmd == "no-direct-script-usage-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-direct-script-usage.py"],
            ns.report,
        )

    if ns.ops_cmd == "directory-budgets-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "scripts/areas/layout/check_scripts_submodules.py", "--threshold", "25"],
            ns.report,
        )

    if ns.ops_cmd == "naming-check":
        return _run_simple_cmd(ctx, ["python3", "scripts/areas/layout/check_ops_script_names.py"], ns.report)

    if ns.ops_cmd == "layer-drift-check":
        return _run_simple_cmd(ctx, ["python3", "scripts/areas/layout/check_layer_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-index":
        cmd = ["python3", "scripts/areas/docs/generate_ops_contracts_doc.py"]
        return _run_simple_cmd(ctx, cmd, ns.report)

    return 2


def configure_ops_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ops", help="ops lint and contracts command surface")
    ops_sub = p.add_subparsers(dest="ops_cmd", required=True)

    check = ops_sub.add_parser("check", help="run canonical ops/check lane")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--fix", action="store_true")

    lint = ops_sub.add_parser("lint", help="run canonical ops lint checks")
    lint.add_argument("--report", choices=["text", "json"], default="text")
    lint.add_argument("--fail-fast", action="store_true")
    lint.add_argument("--emit-artifacts", action="store_true")
    lint.add_argument("--fix", action="store_true")

    for name, help_text in (
        ("surface", "validate or generate ops surface metadata"),
        ("contracts-check", "validate ops contracts index and schema pairs"),
        ("suites-check", "validate ops suite references"),
        ("schema-check", "validate ops schema contracts"),
        ("tool-versions-check", "validate pinned ops tool versions"),
        ("no-direct-script-usage-check", "validate direct ops script usage policy"),
        ("directory-budgets-check", "validate ops-related directory budgets"),
        ("naming-check", "validate ops naming conventions"),
        ("layer-drift-check", "validate cross-layer drift rules"),
        ("contracts-index", "generate ops contracts docs index"),
    ):
        cmd = ops_sub.add_parser(name, help=help_text)
        cmd.add_argument("--report", choices=["text", "json"], default="text")
        cmd.add_argument("--fix", action="store_true")
