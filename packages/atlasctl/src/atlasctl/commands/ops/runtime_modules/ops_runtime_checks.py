from __future__ import annotations

import argparse
import json
import os
import re
import shutil
from collections import defaultdict
from dataclasses import dataclass
from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from typing import Callable

from ...core.context import RunContext
from ...core.fs import ensure_evidence_path
from ...core.process import run_command


@dataclass(frozen=True)
class OpsCheck:
    check_id: str
    description: str
    cmd: list[str]
    actionable: str


def _check(check_id: str, description: str, cmd: list[str], actionable: str) -> OpsCheck:
    return OpsCheck(check_id, description, cmd, actionable)


SELF_CLI = ["python3", "-m", "atlasctl.cli"]


LINT_CHECKS: list[OpsCheck] = [
    _check(
        "ops-run-entrypoints",
        "Ensure ops run entrypoints policy",
        ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_ops_run_entrypoints.py"],
        "Route public ops entrypoints through ops/run/* wrappers.",
    ),
    _check(
        "ops-shell-policy",
        "Ensure ops shell policy",
        ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/runtime/check_ops_shell_policy.py"],
        "Use shared ops shell wrappers and policy-compliant shell structure.",
    ),
    _check(
        "ops-evidence-writes",
        "Ensure ops evidence write policy",
        ["python3", "packages/atlasctl/src/atlasctl/checks/layout/policies/hygiene/check_no_ops_evidence_writes.py"],
        "Write runtime artifacts only under artifacts/evidence or approved allowlist roots.",
    ),
    _check(
        "ops-load-suite-manifest",
        "Validate load suite manifest",
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/load/contracts/validate_suite_manifest.py"],
        "Fix suite schema violations and keep suite manifest aligned with contract.",
    ),
    _check(
        "ops-k8s-surface-generate",
        "Generate k8s test surface",
        [*SELF_CLI, "ops", "k8s-surface-generate", "--report", "text"],
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
        [*SELF_CLI, "ops", "k8s-test-lib-contract", "--report", "text"],
        "Keep k8s test helper library canonical and deduplicated.",
    ),
    _check(
        "ops-k8s-checks-layout",
        "Validate k8s checks layout budget",
        [*SELF_CLI, "ops", "k8s-checks-layout", "--report", "text"],
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
        [*SELF_CLI, "run", "./packages/atlasctl/src/atlasctl/observability/contracts/profiles/check_profile_goldens.py"],
        "Refresh approved profile goldens through the documented update flow.",
    ),
    _check(
        "ops-tool-versions",
        "Validate pinned tool versions",
        ["python3", "packages/atlasctl/src/atlasctl/observability/contracts/governance/check_tool_versions.py", "kind", "kubectl", "helm", "k6"],
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
    result = run_command(cmd, repo_root)
    return result.code, result.combined_output


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
        "tool": "bijux-atlas",
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
        "tool": "bijux-atlas",
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


