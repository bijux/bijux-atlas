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

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path
from ..core.process import run_command


@dataclass(frozen=True)
class OpsCheck:
    check_id: str
    description: str
    cmd: list[str]
    actionable: str


def _check(check_id: str, description: str, cmd: list[str], actionable: str) -> OpsCheck:
    return OpsCheck(check_id, description, cmd, actionable)


SELF_CLI = ["python3", "-m", "bijux_atlas_scripts.cli"]


LINT_CHECKS: list[OpsCheck] = [
    _check(
        "ops-run-entrypoints",
        "Ensure ops run entrypoints policy",
        ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_ops_run_entrypoints.py"],
        "Route public ops entrypoints through ops/run/* wrappers.",
    ),
    _check(
        "ops-shell-policy",
        "Ensure ops shell policy",
        ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_ops_shell_policy.py"],
        "Use shared ops shell wrappers and policy-compliant shell structure.",
    ),
    _check(
        "ops-evidence-writes",
        "Ensure ops evidence write policy",
        ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_no_ops_evidence_writes.py"],
        "Write runtime artifacts only under artifacts/evidence or approved allowlist roots.",
    ),
    _check(
        "ops-load-suite-manifest",
        "Validate load suite manifest",
        [*SELF_CLI, "run", "./packages/bijux-atlas-scripts/src/bijux_atlas_scripts/load/validate_suite_manifest.py"],
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
        [*SELF_CLI, "run", "./packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/check_profile_goldens.py"],
        "Refresh approved profile goldens through the documented update flow.",
    ),
    _check(
        "ops-tool-versions",
        "Validate pinned tool versions",
        ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_tool_versions.py", "kind", "kubectl", "helm", "k6"],
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


def _ops_policy_audit(ctx: RunContext, report_format: str) -> int:
    repo = ctx.repo_root
    env_schema = json.loads((repo / "configs/ops/env.schema.json").read_text(encoding="utf-8"))
    vars_declared = sorted(env_schema.get("variables", {}).keys())
    search_paths = [
        repo / "makefiles/env.mk",
        repo / "makefiles/ops.mk",
        repo / "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/validate_ops_env.py",
        repo / "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/configs/command.py",
        repo / "crates/bijux-atlas-server/src/main.rs",
    ]
    text = "\n".join(p.read_text(encoding="utf-8") for p in search_paths if p.exists())
    violations: list[str] = []
    for var in vars_declared:
        if re.search(rf"\b{re.escape(var)}\b", text) is None:
            violations.append(f"ops env variable `{var}` not reflected in make/scripts usage")
    if "configs/ops/tool-versions.json" not in (repo / "makefiles/ops.mk").read_text(encoding="utf-8"):
        violations.append("ops.mk must reference configs/ops/tool-versions.json")

    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "pass" if not violations else "fail",
        "violations": violations,
    }
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        if violations:
            for v in violations:
                print(f"ops-policy-audit violation: {v}")
        else:
            print("ops policy audit passed")
    return 0 if not violations else 1


def _k8s_checks_layout(repo_root: Path) -> tuple[int, str]:
    checks_dir = repo_root / "ops" / "k8s" / "tests" / "checks"
    errors: list[str] = []
    max_files = 10

    for area in sorted(p for p in checks_dir.iterdir() if p.is_dir() and p.name != "_lib"):
        direct_tests = sorted(area.glob("test_*.sh"))
        has_submodules = any(p.is_dir() for p in area.iterdir())
        if len(direct_tests) > max_files and not has_submodules:
            errors.append(
                f"{area.relative_to(repo_root)} has {len(direct_tests)} test files; max {max_files} without submodules"
            )

    cfg = checks_dir / "config"
    for path in sorted(cfg.glob("test_*.sh")):
        if "config" not in path.name and "envfrom" not in path.name:
            errors.append(f"{path.relative_to(repo_root)} is under config/ but does not look config-related")

    manifest = json.loads((repo_root / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    for test in manifest.get("tests", []):
        groups = {g for g in test.get("groups", []) if isinstance(g, str)}
        if "obs" in groups:
            errors.append(f"{test.get('script')}: ambiguous group `obs` forbidden; use `observability`")

    if errors:
        return 1, "k8s checks layout lint failed\n" + "\n".join(f"- {e}" for e in errors)
    return 0, "k8s checks layout lint passed"


def _k8s_flakes(repo_root: Path) -> tuple[int, str]:
    report = repo_root / "artifacts/ops/k8s/flake-report.json"
    if not report.exists():
        return 0, "flake report missing; skipping"

    payload = json.loads(report.read_text(encoding="utf-8"))
    count = int(payload.get("flake_count", 0))
    if count == 0:
        return 0, "no flakes detected"

    lines = [f"flake detected: {count}"]
    for flake in payload.get("flakes", []):
        lines.append(f"- {flake.get('script')} owner={flake.get('owner')} attempts={flake.get('attempts')}")

    issue_path = repo_root / "artifacts/ops/k8s/flake-issue.md"
    issue_path.parent.mkdir(parents=True, exist_ok=True)
    ttl_days = int(os.environ.get("ATLAS_FLAKE_TTL_DAYS", "14"))
    quarantine_until = (date.today() + timedelta(days=ttl_days)).isoformat()
    body = [
        "# K8s E2E Flake Detected",
        "",
        f"Count: {count}",
        "",
        f"Quarantine TTL: {ttl_days} days (until `{quarantine_until}`)",
        "",
        "## Flakes",
    ]
    for flake in payload.get("flakes", []):
        body.append(f"- `{flake.get('script')}` owner={flake.get('owner')} attempts={flake.get('attempts')}")
    body.append("\nAction: quarantine with TTL in `ops/k8s/tests/manifest.json`.")
    body.append(f"Set `quarantine_until` to `{quarantine_until}` or later for each flaky test.")
    issue_path.write_text("\n".join(body) + "\n", encoding="utf-8")

    if os.environ.get("CI", "").lower() in {"1", "true", "yes"}:
        return 1, "\n".join(lines)
    return 0, "\n".join(lines + ["flake policy warning (non-CI)"])


def _k8s_test_contract(repo_root: Path) -> tuple[int, str]:
    manifest = json.loads((repo_root / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    ownership = json.loads((repo_root / "ops/k8s/tests/ownership.json").read_text(encoding="utf-8"))
    tests = manifest["tests"]
    owners = ownership["owners"]
    errors: list[str] = []

    all_scripts = {t["script"] for t in tests}
    scripts_by_name: dict[str, list[str]] = {}
    for script in all_scripts:
        scripts_by_name.setdefault(Path(script).name, []).append(script)

    for test in tests:
        if "owner" not in test:
            errors.append(f"missing owner in manifest: {test['script']}")
        if "timeout_seconds" not in test:
            errors.append(f"missing timeout_seconds in manifest: {test['script']}")
        groups = test.get("groups")
        if not isinstance(groups, list) or not groups:
            errors.append(f"missing/non-list groups in manifest: {test['script']}")
        efm = test.get("expected_failure_modes")
        if not isinstance(efm, list) or not efm:
            errors.append(f"missing/non-list expected_failure_modes in manifest: {test['script']}")
        if groups != sorted(groups or []):
            errors.append(f"manifest groups must be sorted for deterministic ordering: {test['script']}")

        script_path = repo_root / "ops/k8s/tests" / test["script"]
        if script_path.exists():
            body = script_path.read_text(encoding="utf-8")
            emitted = {
                m.lower()
                for m in re.findall(r"failure_mode\\s*[:=]\\s*([a-z0-9_]+)", body, flags=re.IGNORECASE)
            }
            declared = {m.lower() for m in test.get("expected_failure_modes", []) if isinstance(m, str)}
            undeclared = sorted(emitted - declared)
            if undeclared:
                errors.append(f"script emits undeclared failure_mode(s) {undeclared} for {test['script']}")

    claimed = set()
    for owner, scripts in owners.items():
        for script in scripts:
            resolved = script
            if script not in all_scripts and "/" not in script:
                matches = scripts_by_name.get(script, [])
                if len(matches) == 1:
                    resolved = matches[0]
                elif len(matches) > 1:
                    errors.append(f"ownership map test '{script}' is ambiguous for owner '{owner}': {matches}")
                    continue
            claimed.add(resolved)
            if resolved not in all_scripts:
                errors.append(f"ownership map has unknown test '{resolved}' for owner '{owner}'")

    for script in sorted(all_scripts):
        if script not in claimed:
            errors.append(f"manifest test not in ownership map: {script}")

    for test in tests:
        if test["owner"] not in owners:
            errors.append(f"manifest owner '{test['owner']}' not declared in ownership map for {test['script']}")

    if errors:
        return 1, "\n".join(errors)
    return 0, "k8s test contract check passed"


def _k8s_test_lib(repo_root: Path) -> tuple[int, str]:
    lib_dir = repo_root / "ops/k8s/tests/checks/_lib"
    files = sorted(p for p in lib_dir.glob("*.sh") if p.is_file())
    if len(files) > 10:
        return 1, f"k8s test lib contract failed: {lib_dir.relative_to(repo_root)} has {len(files)} files (max 10)"
    for path in files:
        text = path.read_text(encoding="utf-8")
        if "k8s-test-common.sh" not in text:
            return (
                1,
                f"k8s test lib contract failed: {path.relative_to(repo_root)} must source canonical k8s-test-common.sh",
            )
    return 0, "k8s test lib contract passed"


def _k8s_surface_generate(repo_root: Path) -> tuple[int, str]:
    manifest = json.loads((repo_root / "ops/k8s/tests/manifest.json").read_text(encoding="utf-8"))
    suites = json.loads((repo_root / "ops/k8s/tests/suites.json").read_text(encoding="utf-8"))
    tests = sorted(manifest.get("tests", []), key=lambda x: x["script"])
    by_group: dict[str, list[str]] = defaultdict(list)
    for test in tests:
        for group in sorted(test.get("groups", [])):
            by_group[group].append(test["script"])

    out_index = repo_root / "ops/k8s/tests/INDEX.md"
    out_doc = repo_root / "docs/_generated/k8s-test-surface.md"

    lines = ["# K8s Tests Index", "", "Generated from `ops/k8s/tests/manifest.json`.", "", "## Groups"]
    for group in sorted(by_group):
        lines.append(f"- `{group}` ({len(by_group[group])})")
    lines.extend(["", "## Tests"])
    for test in tests:
        lines.append(f"- `{test['script']}` groups={','.join(test.get('groups', []))} owner={test.get('owner','unknown')}")
    out_index.write_text("\n".join(lines) + "\n", encoding="utf-8")

    suite_map = {s["id"]: sorted(s.get("groups", [])) for s in suites.get("suites", [])}
    doc = ["# K8s Test Surface", "", "Generated from `ops/k8s/tests/manifest.json` and `ops/k8s/tests/suites.json`.", "", "## Suites"]
    for sid in sorted(suite_map):
        doc.append(f"- `{sid}` groups={','.join(suite_map[sid]) if suite_map[sid] else '*'}")
    doc.extend(["", "## Group -> Tests"])
    for group in sorted(by_group):
        doc.append(f"### `{group}`")
        for script in sorted(by_group[group]):
            doc.append(f"- `{script}`")
        doc.append("")
    out_doc.write_text("\n".join(doc).rstrip() + "\n", encoding="utf-8")
    return 0, f"generated {out_index.relative_to(repo_root)} and {out_doc.relative_to(repo_root)}"


def _ops_clean_generated(ctx: RunContext, report_format: str, force: bool) -> int:
    generated_root = ctx.repo_root / "ops" / "_generated"
    if not generated_root.exists():
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "run_id": ctx.run_id,
            "status": "pass",
            "message": "ops/_generated does not exist",
        }
        if report_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(payload["message"])
        return 0

    probe = run_command(["git", "check-ignore", "-q", "ops/_generated/probe.file"], ctx.repo_root)
    ignored = probe.code == 0
    if not ignored and not force:
        message = "refusing to clean ops/_generated because it is not ignored; pass --force to override"
        if report_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "bijux-atlas",
                        "run_id": ctx.run_id,
                        "status": "fail",
                        "message": message,
                    },
                    sort_keys=True,
                )
            )
        else:
            print(message)
        return 1

    removed: list[str] = []
    for child in sorted(generated_root.iterdir()):
        removed.append(child.name)
        if child.is_dir():
            shutil.rmtree(child)
        else:
            child.unlink()
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "pass",
        "path": str(generated_root.relative_to(ctx.repo_root)),
        "removed_entries": removed,
    }
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"cleaned {payload['path']} ({len(removed)} entries removed)")
    return 0


def run_ops_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.ops_cmd == "check":
        steps = [
            [*SELF_CLI, "ops", "lint", "--report", ns.report, "--emit-artifacts"],
            [*SELF_CLI, "ops", "contracts-check", "--report", ns.report],
            [*SELF_CLI, "ops", "suites-check", "--report", ns.report],
            [*SELF_CLI, "ops", "schema-check", "--report", ns.report],
            ["env", "CACHE_STATUS_STRICT=0", "make", "-s", "ops-cache-status"],
            ["make", "-s", "pins/check"],
            [*SELF_CLI, "ops", "surface", "--report", ns.report],
            ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_ops_index_surface.py"],
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
                ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/generate_ops_surface_meta.py"],
                [*SELF_CLI, "docs", "generate", "--report", "text"],
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
            return _run_simple_cmd(ctx, ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/generate_ops_surface_meta.py"], ns.report)
        return _run_simple_cmd(ctx, ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_ops_surface_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-check":
        return _run_simple_cmd(ctx, ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/validate_ops_contracts.py"], ns.report)

    if ns.ops_cmd == "suites-check":
        return _run_simple_cmd(
            ctx,
            ["python3", "ops/_lint/no-orphan-suite.py"],
            ns.report,
        )

    if ns.ops_cmd == "schema-check":
        return _run_simple_cmd(ctx, ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/validate_ops_contracts.py"], ns.report)

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
            ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_scripts_submodules.py", "--threshold", "25"],
            ns.report,
        )

    if ns.ops_cmd == "naming-check":
        return _run_simple_cmd(ctx, ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_ops_script_names.py"], ns.report)

    if ns.ops_cmd == "layer-drift-check":
        return _run_simple_cmd(ctx, ["python3", "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_layer_drift.py"], ns.report)

    if ns.ops_cmd == "contracts-index":
        cmd = [*SELF_CLI, "docs", "generate", "--report", "text"]
        return _run_simple_cmd(ctx, cmd, ns.report)
    if ns.ops_cmd == "policy-audit":
        return _ops_policy_audit(ctx, ns.report)
    if ns.ops_cmd == "k8s-flakes-check":
        code, output = _k8s_flakes(ctx.repo_root)
        print(output)
        return code
    if ns.ops_cmd == "k8s-test-contract":
        code, output = _k8s_test_contract(ctx.repo_root)
        print(output)
        return code
    if ns.ops_cmd == "k8s-test-lib-contract":
        code, output = _k8s_test_lib(ctx.repo_root)
        print(output)
        return code
    if ns.ops_cmd == "k8s-checks-layout":
        code, output = _k8s_checks_layout(ctx.repo_root)
        print(output)
        return code
    if ns.ops_cmd == "k8s-surface-generate":
        code, output = _k8s_surface_generate(ctx.repo_root)
        print(output)
        return code
    if ns.ops_cmd == "clean-generated":
        return _ops_clean_generated(ctx, ns.report, ns.force)

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
        ("policy-audit", "validate ops policy configs reflected in ops usage"),
        ("k8s-surface-generate", "generate k8s test surface docs from manifest"),
        ("k8s-checks-layout", "validate k8s checks layout budget"),
        ("k8s-test-lib-contract", "validate k8s tests checks/_lib helper contract"),
        ("k8s-flakes-check", "evaluate k8s flake report and quarantine policy"),
        ("k8s-test-contract", "validate k8s test manifest ownership/contract"),
        ("clean-generated", "remove runtime evidence files under ops/_generated"),
    ):
        cmd = ops_sub.add_parser(name, help=help_text)
        cmd.add_argument("--report", choices=["text", "json"], default="text")
        cmd.add_argument("--fix", action="store_true")
        if name == "clean-generated":
            cmd.add_argument("--force", action="store_true")
