from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable

from ....core.context import RunContext
from ....core.fs import ensure_evidence_path

@dataclass(frozen=True)
class MakeCheck:
    check_id: str
    description: str
    cmd: list[str]
    actionable: str


def _check(check_id: str, description: str, script: str, actionable: str) -> MakeCheck:
    return MakeCheck(check_id, description, ["python3", script], actionable)


def _check_cmd(check_id: str, description: str, cmd: list[str], actionable: str) -> MakeCheck:
    return MakeCheck(check_id, description, cmd, actionable)


CHECKS: list[MakeCheck] = [
    _check(
        "public-surface",
        "Validate public make surface contract",
        "packages/atlasctl/src/atlasctl/checks/layout/public_surface/checks/check_public_surface.py",
        "Run make inventory and keep public targets in SSOT.",
    ),
    _check(
        "no-dead-entrypoints",
        "Validate referenced scripts and targets exist",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/hygiene/check_no_dead_entrypoints.py",
        "Update stale references or remove dead entrypoints.",
    ),
    _check(
        "no-orphan-docs-refs",
        "Validate documented commands exist",
        "packages/atlasctl/src/atlasctl/checks/layout/docs/check_no_orphan_docs_refs.py",
        "Fix docs references or add missing command help coverage.",
    ),
    _check(
        "no-orphan-configs",
        "Validate config files are referenced or declared internal",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/orphans/check_no_orphan_configs.py",
        "Add docs/contract references or annotate internal config ownership.",
    ),
    _check(
        "no-orphan-owners",
        "Validate ownership coverage",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/orphans/check_no_orphan_owners.py",
        "Add missing owners for areas, paths, and command surfaces.",
    ),
    _check_cmd(
        "docs-public-surface",
        "Validate docs public surface consistency",
        ["python3", "-m", "atlasctl.cli", "docs", "public-surface-check", "--report", "json"],
        "Regenerate docs/_generated surfaces and align nav references.",
    ),
    _check_cmd(
        "suite-id-docs",
        "Validate suite id docs coverage",
        ["python3", "-m", "atlasctl.cli", "docs", "suite-id-docs-check", "--report", "json"],
        "Document missing suite ids or remove stale suite references.",
    ),
    _check(
        "ci-entrypoints",
        "Validate CI workflows only call allowed targets",
        "packages/atlasctl/src/atlasctl/checks/layout/workflows/check_ci_entrypoints.py",
        "Update workflow jobs to call approved public targets only.",
    ),
    _check(
        "workflow-command-bans",
        "Validate workflows ban direct cargo and internal make invocations",
        "packages/atlasctl/src/atlasctl/checks/layout/workflows/check_workflow_command_bans.py",
        "Use atlasctl commands or approved public wrappers in workflows.",
    ),
    _check(
        "ci-legacy-target-cutoff",
        "Validate CI workflows have no legacy targets after cutoff date",
        "packages/atlasctl/src/atlasctl/checks/layout/workflows/check_ci_legacy_target_cutoff.py",
        "Replace legacy make target calls in workflows with canonical atlasctl-backed targets before cutoff date.",
    ),
    _check(
        "ci-write-scope",
        "Validate ci wrappers avoid direct writes outside isolate/evidence",
        "packages/atlasctl/src/atlasctl/checks/layout/workflows/check_ci_writes_scoped.py",
        "Keep makefiles/ci.mk wrapper-only and route all writes through atlasctl-managed isolate/evidence outputs.",
    ),
    _check_cmd(
        "dev-ci-target-map",
        "Validate cargo/ci make targets are mapped to atlasctl DEV/CI intents",
        ["python3", "-m", "atlasctl.cli", "make", "dev-ci-target-map", "--check", "--json"],
        "Map every cargo.mk/ci.mk target to one stable atlasctl intent and declare aliases explicitly.",
    ),
    _check(
        "help-excludes-internal",
        "Ensure help excludes internal targets",
        "packages/atlasctl/src/atlasctl/checks/layout/docs/check_help_excludes_internal.py",
        "Move internal targets out of public help rendering.",
    ),
    _check(
        "public-target-ownership",
        "Ensure public target ownership coverage",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_target_ownership.py",
        "Add missing target owners in makefiles/ownership.json.",
    ),
    _check(
        "public-target-docs",
        "Ensure public target docs coverage",
        "packages/atlasctl/src/atlasctl/checks/layout/public_surface/checks/check_public_targets_documented.py",
        "Document missing targets under docs/_generated/make-targets.md.",
    ),
    _check(
        "public-target-budget",
        "Ensure public target budget",
        "packages/atlasctl/src/atlasctl/checks/layout/public_surface/checks/check_public_target_budget.py",
        "Trim public targets or increase budget with governance approval.",
    ),
    _check(
        "public-target-descriptions",
        "Validate public target descriptions",
        "packages/atlasctl/src/atlasctl/checks/layout/public_surface/checks/check_public_target_descriptions.py",
        "Add concise help descriptions for all public targets.",
    ),
    _check(
        "public-target-aliases",
        "Validate public target aliases",
        "packages/atlasctl/src/atlasctl/checks/layout/public_surface/checks/check_public_target_aliases.py",
        "Remove undocumented aliases or document supported aliases.",
    ),
    _check(
        "internal-target-doc-refs",
        "Validate internal targets are not in docs",
        "packages/atlasctl/src/atlasctl/checks/layout/docs/check_internal_targets_not_in_docs.py",
        "Replace internal target references in docs with public targets.",
    ),
    _check(
        "makefile-boundaries",
        "Validate makefile target boundaries",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_makefile_target_boundaries.py",
        "Keep top-level/public and internal target boundaries strict.",
    ),
    _check(
        "makefiles-contract",
        "Validate makefiles contract",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_makefiles_contract.py",
        "Regenerate makefile contract artifacts and align file ownership.",
    ),
    _check(
        "makefiles-headers",
        "Validate makefile header contract",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_makefile_headers.py",
        "Add or correct required makefile scope headers.",
    ),
    _check(
        "ci-mk-size-budget",
        "Validate ci.mk size budget",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_ci_mk_size_budget.py",
        "Keep makefiles/ci.mk as a thin wrapper and move execution logic into atlasctl dev ci commands.",
    ),
    _check(
        "makefiles-index-drift",
        "Validate makefiles index drift",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/index/check_makefiles_index_drift.py",
        "Regenerate makefile index docs and commit deterministic output.",
    ),
    _check(
        "make-targets-catalog-drift",
        "Validate make targets catalog drift",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_targets_catalog_drift.py",
        "Regenerate make targets catalog and commit updates.",
    ),
    _check(
        "cargo-dev-metadata",
        "Validate cargo-dev metadata consistency",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/policies/check_cargo_dev_metadata.py",
        "Align cargo-dev metadata with declared make targets.",
    ),
    _check(
        "root-no-cargo-dev-deps",
        "Validate root has no cargo-dev deps",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/root/check_root_no_cargo_dev_deps.py",
        "Move cargo-dev-only dependencies out of the root lane.",
    ),
    _check(
        "cargo-invocation-scope",
        "Validate cargo invocation scoping",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/policies/check_cargo_invocations_scoped.py",
        "Use lane wrappers to scope cargo invocations correctly.",
    ),
    _check(
        "cargo-mk-wrapper-purity",
        "Validate cargo.mk contains wrappers only (no cargo/python3/rm logic)",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_cargo_mk_wrapper_purity.py",
        "Keep makefiles/cargo.mk as pure atlasctl delegation wrappers only.",
    ),
    _check(
        "make-wrapper-forbidden-tokens",
        "Validate wrapper makefiles contain no direct tool tokens",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_wrapper_forbidden_tokens.py",
        "Keep wrapper recipes as atlasctl-only delegation (no cargo/python3/kubectl/etc).",
    ),
    _check(
        "make-wrapper-no-multiline",
        "Validate wrapper make targets use single-line recipes",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_wrapper_no_multiline_recipes.py",
        "Use single-line `@./bin/atlasctl ...` recipes in wrapper makefiles.",
    ),
    _check(
        "make-wrapper-owners",
        "Validate wrapper make targets have ownership metadata",
        "packages/atlasctl/src/atlasctl/checks/layout/makefiles/checks/check_make_wrapper_target_owners.py",
        "Add owner/area entries in makefiles/ownership.json for wrapper targets.",
    ),
    _check(
        "root-diff-alarm",
        "Validate root diff alarm contract",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/root/check_root_diff_alarm.py",
        "Update root diff alarm allowlist or reduce root-level churn.",
    ),
    _check(
        "help-output-determinism",
        "Validate help output determinism",
        "packages/atlasctl/src/atlasctl/checks/layout/docs/check_help_output_determinism.py",
        "Remove nondeterministic ordering from help rendering sources.",
    ),
    _check(
        "help-snapshot",
        "Validate help snapshot",
        "packages/atlasctl/src/atlasctl/checks/layout/docs/check_help_snapshot.py",
        "Update help snapshot intentionally after reviewing target surface changes.",
    ),
    _check(
        "no-legacy-target-names",
        "Validate no legacy target names",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/deprecation/check_no_legacy_target_names.py",
        "Rename or delete legacy targets; do not keep compatibility aliases.",
    ),
    _check(
        "root-mk-size-budget",
        "Validate root.mk size budget",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/root/check_root_mk_size_budget.py",
        "Move lane-specific logic to dedicated makefiles to stay within budget.",
    ),
    _check(
        "root-makefile-hygiene",
        "Validate root makefile hygiene",
        "packages/atlasctl/src/atlasctl/checks/layout/policies/root/check_root_makefile_hygiene.py",
        "Fix ordering, phony coverage, and structural hygiene issues in root.mk.",
    ),
]


def _surface_payload() -> dict[str, object]:
    owners = load_ownership()
    entries = public_entries()
    targets = []
    for item in entries:
        name = str(item.get("name", ""))
        meta = owners.get(name, {}) if isinstance(owners, dict) else {}
        targets.append(
            {
                "name": name,
                "description": str(item.get("description", "")),
                "area": str(item.get("area", "")),
                "lanes": item.get("lanes", []),
                "owner": str(meta.get("owner", "unknown")),
            }
        )
    return {"schema_version": 1, "targets": targets}


def _contracts_report(
    checks: list[dict[str, object]],
    started_at: str,
    ended_at: str,
    run_id: str,
) -> dict[str, object]:
    failed = [c for c in checks if c.get("status") == "fail"]
    return {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": run_id,
        "status": "fail" if failed else "pass",
        "started_at": started_at,
        "ended_at": ended_at,
        "checks": checks,
        "failed_count": len(failed),
        "total_count": len(checks),
    }


def _run_check(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output.strip()


def run_contracts_check(
    ctx: RunContext,
    fail_fast: bool,
    emit_artifacts: bool,
    as_json: bool,
    runner: Callable[[list[str], Path], tuple[int, str]] = _run_check,
) -> int:
    repo_root = ctx.repo_root
    started_at = datetime.now(timezone.utc).isoformat()
    results: list[dict[str, object]] = []

    for check in CHECKS:
        code, output = runner(check.cmd, repo_root)
        status = "pass" if code == 0 else "fail"
        item = {
            "id": check.check_id,
            "description": check.description,
            "status": status,
            "command": " ".join(check.cmd),
            "actionable": check.actionable,
        }
        if status == "fail":
            item["error"] = output
        results.append(item)
        if fail_fast and status == "fail":
            break

    ended_at = datetime.now(timezone.utc).isoformat()
    payload = _contracts_report(results, started_at, ended_at, ctx.run_id)

    if emit_artifacts:
        out = ensure_evidence_path(
            ctx,
            ctx.evidence_root / "make" / ctx.run_id / "contracts-check.json",
        )
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    schema_path = repo_root / "configs/contracts/make-contracts-check-output.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    import jsonschema

    jsonschema.validate(payload, schema)

    if as_json:
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            "make contracts-check: "
            f"status={payload['status']} "
            f"checks={payload['total_count']} "
            f"failed={payload['failed_count']}"
        )
        for row in payload["checks"]:
            if row["status"] == "fail":
                first_line = (
                    row.get("error", "").splitlines()[:1][0]
                    if row.get("error")
                    else "check failed"
                )
                print(f"- FAIL {row['id']}: {first_line}")
                print(f"  fix: {row['actionable']}")
    return 0 if payload["status"] == "pass" else 1
