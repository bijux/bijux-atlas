from __future__ import annotations

import datetime as dt
import fnmatch
import json
import os
import re
from pathlib import Path

from .enforcement import (
    check_ci_pr_lane_fast_only,
    check_make_ci_entrypoints_contract,
    check_make_index_drift_contract,
    check_make_no_orphan_docs_refs,
    check_make_no_direct_artifact_writes,
    check_make_no_bypass_atlasctl_without_allowlist,
    check_make_no_direct_script_exec_drift,
    check_make_no_direct_python_only_atlasctl,
    check_make_no_direct_bash_ops,
    check_make_no_legacy_script_aliases,
    check_make_no_direct_scripts_only_atlasctl,
    check_make_lane_reports_via_atlasctl_reporting,
    check_make_no_atlasctl_run_ops_run,
    check_make_product_mk_wrapper_contract,
    check_make_product_migration_complete_no_ops_run,
    check_ci_workflows_call_make_and_make_calls_atlasctl,
    check_public_make_targets_map_to_atlasctl,
    check_make_public_targets_documented,
    check_make_wrapper_purity,
    check_makefiles_wrappers_only_all,
    check_make_no_python_module_invocation,
    check_make_root_budget,
    check_make_target_boundaries_enforced,
    check_make_target_ownership_complete,
    check_workflows_reference_known_suites,
    check_policies_bypass_no_new_files,
    check_policies_bypass_all_entries_have_owner_issue_expiry,
    check_policies_bypass_expiry_not_past,
    check_policies_bypass_expiry_max_horizon,
    check_policies_bypass_no_blank_justifications,
    check_policies_bypass_issue_id_format,
    check_policies_bypass_owner_in_owners_registry,
    check_policies_bypass_removal_plan_required,
    check_policies_bypass_scope_valid,
    check_policies_bypass_policy_name_known,
    check_policies_bypass_schema_valid,
    check_policies_bypass_inventory_present,
    check_policies_bypass_inventory_schema_valid,
    check_policies_bypass_inventory_deterministic,
    check_policies_bypass_has_owner,
    check_policies_bypass_has_expiry,
    check_policies_bypass_has_reason,
    check_policies_bypass_has_ticket_or_doc_ref,
    check_policies_bypass_budget_trend,
    check_policies_bypass_new_entries_forbidden,
    check_policies_bypass_entry_paths_exist,
    check_policies_bypass_entry_matches_nothing,
    check_policies_bypass_entry_matches_too_broad,
    check_policies_bypass_budget,
    check_policies_bypass_readme_complete,
    check_policies_bypass_readme_sorted,
    collect_bypass_inventory,
)
from ....repo.domains.forbidden_adjectives import check_forbidden_adjectives
from ....repo.native import (
    check_make_command_allowlist,
    check_make_forbidden_paths,
    check_make_help,
    check_make_no_direct_python_script_invocations,
    check_make_no_duplicate_all_variants,
    check_make_scripts_references,
    check_make_target_names_no_banned_adjectives,
    check_make_wrapper_no_direct_cargo,
    check_make_wrapper_no_env_side_effects,
    check_make_wrapper_no_multiline_recipes,
    check_make_wrapper_only_calls_bin_atlasctl,
    check_make_wrapper_phony_complete,
    check_make_wrapper_shell_is_sh,
    check_make_wrapper_target_budget,
)
from ....core.base import CheckDef

_NETWORK_ALLOWLIST = Path("configs/policy/shell-network-fetch-allowlist.txt")
_PROBES_ALLOWLIST = Path("configs/policy/shell-probes-allowlist.txt")
_DEPENDENCY_EXCEPTIONS = Path("configs/policy/dependency-exceptions.json")
_MILESTONES = Path("configs/policy/bypass-removal-milestones.json")
_BYPASS_COUNT_BASELINE = Path("configs/policy/bypass-count-baseline.json")
_ADJECTIVE_ALLOWLIST = Path("configs/policy/forbidden-adjectives-allowlist.txt")
_BUDGET_APPROVAL = Path("configs/policy/budget-loosening-approval.json")
_BYPASS_TEST_COVERAGE = Path("configs/policy/bypass-test-coverage.json")
_ISSUE_RE = re.compile(r"^ISSUE-[A-Z0-9-]+$")


def _allowlist_inline_meta(path: Path) -> list[str]:
    if not path.exists():
        return []
    errors: list[str] = []
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "#" not in raw:
            errors.append(f"{path.as_posix()}:{lineno}: entry must include inline metadata comment `# owner=<id>; why=<reason>`")
            continue
        _, comment = raw.split("#", 1)
        if "owner=" not in comment.lower() or "why=" not in comment.lower():
            errors.append(f"{path.as_posix()}:{lineno}: inline metadata must include owner= and why=")
    return errors


def check_policies_shell_network_fetch_allowlist_inline_meta(repo_root: Path) -> tuple[int, list[str]]:
    errors = _allowlist_inline_meta(repo_root / _NETWORK_ALLOWLIST)
    return (0 if not errors else 1), sorted(errors)


def check_policies_shell_probes_allowlist_inline_meta(repo_root: Path) -> tuple[int, list[str]]:
    errors = _allowlist_inline_meta(repo_root / _PROBES_ALLOWLIST)
    return (0 if not errors else 1), sorted(errors)


def check_policies_adjectives_repo_clean(repo_root: Path) -> tuple[int, list[str]]:
    return check_forbidden_adjectives(repo_root)


def check_policies_adjective_allowlist_budget(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _ADJECTIVE_ALLOWLIST
    entries = [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip() and not line.strip().startswith("#")] if path.exists() else []
    return (0 if not entries else 1), ([] if not entries else [f"{_ADJECTIVE_ALLOWLIST.as_posix()}: must be empty; found {len(entries)} entries"])


def check_policies_bypass_files_scoped(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    scan_roots = [repo_root / "configs", repo_root / "packages/atlasctl/src/atlasctl", repo_root / "makefiles", repo_root / "ops", repo_root / "scripts", repo_root / ".github"]
    for base in scan_roots:
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            if path.suffix == ".pyc" or "__pycache__" in path.parts:
                continue
            if not any(fnmatch.fnmatch(path.name, pattern) for pattern in ("*allowlist*", "*relax*", "*exceptions*", "*ratchet*")):
                continue
            rel = path.relative_to(repo_root).as_posix()
            if rel.startswith("configs/policy/") or rel.startswith("configs/docs/") or rel.startswith("configs/ops/") or rel.startswith("configs/_schemas/") or rel.startswith("configs/layout/") or rel.startswith("configs/make/") or rel.startswith("configs/repo/") or rel.startswith("configs/security/") or rel.startswith("ops/_artifacts/") or rel.startswith("ops/_lint/") or rel.startswith("ops/_meta/") or rel.startswith("ops/_schemas/") or rel.startswith("ops/vendor/layout-checks/") or rel.startswith("packages/atlasctl/src/atlasctl/checks/layout/makefiles/policies/"):
                continue
            if rel.startswith("packages/atlasctl/tests/"):
                continue
            errors.append(f"bypass-like file outside configs/policy: {rel}")
    return (0 if not errors else 1), errors


def check_policies_no_inline_bypass_entries(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in sorted((repo_root / "packages/atlasctl/src").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/checks/domains/policies/make/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if re.search(r'["\']owner["\']\s*:\s*', text) and re.search(r'["\']issue_id["\']\s*:\s*', text) and re.search(r'["\']removal_plan["\']\s*:\s*', text) and re.search(r'["\']expiry["\']\s*:\s*', text):
            errors.append(f"{rel}: potential inline bypass metadata detected; move bypasses to configs/policy/*")
    return (0 if not errors else 1), errors


def check_policies_tests_bypass_dependency_marked(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in sorted((repo_root / "packages/atlasctl/tests").rglob("test_*.py")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "configs/policy/" not in text:
            continue
        if "# BYPASS_TEST_OK" not in text:
            errors.append(f"{path.relative_to(repo_root).as_posix()}: test depends on bypass files without # BYPASS_TEST_OK marker")
    return (0 if not errors else 1), errors


def check_policies_bypass_usage_heatmap(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    by_source: dict[str, int] = {}
    by_policy: dict[str, int] = {}
    for row in payload.get("entries", []):
        if not isinstance(row, dict):
            continue
        source = str(row.get("source", "")).strip()
        policy = str(row.get("policy_name", "")).strip() or "(unknown)"
        by_source[source] = by_source.get(source, 0) + 1
        by_policy[policy] = by_policy.get(policy, 0) + 1
    _ = {"schema_version": 1, "by_source": by_source, "by_policy": by_policy}
    return 0, []


def check_policies_bypass_removal_milestones_defined(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _MILESTONES
    if not path.exists():
        return 1, [f"missing milestone file: {_MILESTONES.as_posix()}"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = payload.get("milestones", []) if isinstance(payload, dict) else []
    errors: list[str] = []
    if not rows:
        errors.append("bypass removal milestones must contain at least one milestone")
    for row in rows:
        if not isinstance(row, dict):
            errors.append("invalid milestone entry")
            continue
        if not str(row.get("id", "")).strip():
            errors.append("milestone missing id")
        if not str(row.get("target_date", "")).strip():
            errors.append(f"milestone {row.get('id', '(unknown)')}: missing target_date")
        if not isinstance(row.get("bypass_ids", []), list) or not row.get("bypass_ids"):
            errors.append(f"milestone {row.get('id', '(unknown)')}: bypass_ids must be non-empty list")
    return (0 if not errors else 1), errors


def check_policies_bypass_count_nonincreasing(repo_root: Path) -> tuple[int, list[str]]:
    baseline_path = repo_root / _BYPASS_COUNT_BASELINE
    if not baseline_path.exists():
        return 1, [f"missing baseline file: {_BYPASS_COUNT_BASELINE.as_posix()}"]
    baseline = int(json.loads(baseline_path.read_text(encoding="utf-8")).get("max_entries", 0))
    current = int(collect_bypass_inventory(repo_root).get("entry_count", 0))
    return ((0, []) if current <= baseline else (1, [f"bypass entry count regressed: current={current} baseline={baseline}"]))


def check_policies_bypass_hard_gate(repo_root: Path) -> tuple[int, list[str]]:
    """
    Hard-gate milestone: when ATLASCTL_BYPASS_HARD_FAIL=1, any bypass entry fails.
    Default remains non-blocking until explicitly flipped in CI.
    """
    enabled = str(os.environ.get("ATLASCTL_BYPASS_HARD_FAIL", "")).strip().lower() in {"1", "true", "yes", "on"}
    if not enabled:
        enabled = str(os.environ.get("ATLASCTL_BYPASS_STRICT_MODE", "")).strip().lower() in {"1", "true", "yes", "on"}
    if not enabled:
        return 0, []
    current = int(collect_bypass_inventory(repo_root).get("entry_count", 0))
    if current == 0:
        return 0, []
    return 1, [f"bypass hard gate enabled: entry_count={current} must be zero"]


def check_policies_bypass_mainline_strict_mode(repo_root: Path) -> tuple[int, list[str]]:
    """
    Alias for future "no relaxations in mainline" mode.
    Enabled via ATLASCTL_BYPASS_STRICT_MODE (or legacy ATLASCTL_BYPASS_HARD_FAIL).
    """
    return check_policies_bypass_hard_gate(repo_root)


def check_policies_bypass_has_test_coverage(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _BYPASS_TEST_COVERAGE
    if not path.exists():
        return 1, [f"missing coverage registry: {_BYPASS_TEST_COVERAGE.as_posix()}"]
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return 1, [f"{_BYPASS_TEST_COVERAGE.as_posix()}: invalid json: {exc}"]
    rows = payload.get("coverage", []) if isinstance(payload, dict) else []
    if not isinstance(rows, list) or not rows:
        return 1, [f"{_BYPASS_TEST_COVERAGE.as_posix()}: coverage must contain non-empty `coverage` list"]

    source_rows: dict[str, dict[str, object]] = {}
    errors: list[str] = []
    for idx, row in enumerate(rows, start=1):
        if not isinstance(row, dict):
            errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: coverage[{idx}] must be object")
            continue
        source = str(row.get("source", "")).strip()
        tests = row.get("tests", [])
        if not source:
            errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: coverage[{idx}] missing source")
            continue
        if source in source_rows:
            errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: duplicate source coverage entry `{source}`")
            continue
        if not isinstance(tests, list) or not tests:
            errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: {source}: tests must be non-empty list")
            continue
        for t in tests:
            rel = str(t).strip()
            if not rel:
                errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: {source}: blank test path")
                continue
            if not (repo_root / rel).exists():
                errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: {source}: missing test path `{rel}`")
        source_rows[source] = row

    inventory = collect_bypass_inventory(repo_root)
    entries = [row for row in inventory.get("entries", []) if isinstance(row, dict)]
    by_source: dict[str, list[dict[str, object]]] = {}
    for row in entries:
        source = str(row.get("source", "")).strip()
        if not source:
            continue
        by_source.setdefault(source, []).append(row)

    for source, source_entries in sorted(by_source.items()):
        if source not in source_rows:
            errors.append(f"bypass source has no test coverage declaration: {source}")
            continue
        declared = source_rows[source]
        entry_keys = declared.get("entry_keys", ["*"])
        if not isinstance(entry_keys, list) or not entry_keys:
            errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: {source}: entry_keys must be non-empty list when present")
            continue
        patterns = [str(item).strip() for item in entry_keys if str(item).strip()]
        if not patterns:
            errors.append(f"{_BYPASS_TEST_COVERAGE.as_posix()}: {source}: entry_keys contain no usable patterns")
            continue
        uncovered: list[str] = []
        for entry in source_entries:
            key = str(entry.get("key", "")).strip()
            if not key:
                continue
            if any(fnmatch.fnmatch(key, patt) for patt in patterns):
                continue
            uncovered.append(key)
        for key in sorted(uncovered)[:20]:
            errors.append(f"{source}: bypass entry has no declared test coverage: {key}")
        if len(uncovered) > 20:
            errors.append(f"{source}: {len(uncovered) - 20} more bypass entries without declared test coverage")
    return (0 if not errors else 1), sorted(errors)


def check_policies_bypass_ids_unique(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    seen: dict[str, str] = {}
    errors: list[str] = []
    for row in payload.get("entries", []):
        if not isinstance(row, dict):
            continue
        key = str(row.get("key", "")).strip()
        source = str(row.get("source", "")).strip()
        if not key or not key.startswith(("ATLAS-", "ISSUE-")):
            continue
        if key in seen and seen[key] != source:
            errors.append(f"duplicate bypass id/key `{key}` in {source} and {seen[key]}")
        else:
            seen[key] = source
    return (0 if not errors else 1), sorted(errors)


def check_policies_dependency_exceptions_remove_by_issue(repo_root: Path) -> tuple[int, list[str]]:
    payload = json.loads((repo_root / _DEPENDENCY_EXCEPTIONS).read_text(encoding="utf-8")) if (repo_root / _DEPENDENCY_EXCEPTIONS).exists() else {}
    errors: list[str] = []
    for list_key in ("undeclared_import_allowlist", "optional_dependency_usage_allowlist", "internal_third_party_allowlist"):
        for idx, row in enumerate(payload.get(list_key, []) if isinstance(payload, dict) else [], start=1):
            if not isinstance(row, dict):
                continue
            issue = str(row.get("issue_id", "")).strip()
            remove_by = str(row.get("remove_by", "")).strip()
            if not _ISSUE_RE.match(issue):
                errors.append(f"{_DEPENDENCY_EXCEPTIONS.as_posix()}:{list_key}[{idx}]: invalid or missing issue_id")
            if not remove_by:
                errors.append(f"{_DEPENDENCY_EXCEPTIONS.as_posix()}:{list_key}[{idx}]: missing remove_by")
    return (0 if not errors else 1), errors


def check_policies_budget_relaxation_requires_approval(repo_root: Path) -> tuple[int, list[str]]:
    relax = json.loads((repo_root / "configs/policy/budget-relaxations.json").read_text(encoding="utf-8"))
    exceptions = relax.get("exceptions", []) if isinstance(relax, dict) else []
    if not exceptions:
        return 0, []
    approval = json.loads((repo_root / _BUDGET_APPROVAL).read_text(encoding="utf-8")) if (repo_root / _BUDGET_APPROVAL).exists() else {}
    approved = bool(approval.get("approved", False))
    approval_id = str(approval.get("approval_id", "")).strip()
    errors: list[str] = []
    if not approved:
        errors.append("budget relaxations present but budget-loosening-approval.json approved=false")
    if not approval_id:
        errors.append("budget relaxations present but budget-loosening-approval.json approval_id is empty")
    return (0 if not errors else 1), errors


def check_policies_budget_approval_time_bounded(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _BUDGET_APPROVAL
    if not path.exists():
        return 1, [f"missing {_BUDGET_APPROVAL.as_posix()}"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    expiry_raw = str(payload.get("expiry", "")).strip()
    if not expiry_raw:
        return 1, [f"{_BUDGET_APPROVAL.as_posix()}: missing expiry"]
    try:
        expiry = dt.date.fromisoformat(expiry_raw)
    except ValueError:
        return 1, [f"{_BUDGET_APPROVAL.as_posix()}: invalid expiry `{expiry_raw}`"]
    today = dt.date.today()
    if expiry < today:
        return 1, [f"{_BUDGET_APPROVAL.as_posix()}: approval expired on {expiry_raw}"]
    if (expiry - today).days > 365:
        return 1, [f"{_BUDGET_APPROVAL.as_posix()}: expiry horizon exceeds 365 days"]
    return 0, []

CHECKS: tuple[CheckDef, ...] = (
    CheckDef(
        "make.no_direct_scripts_only_atlasctl",
        "make",
        "forbid direct script path invocations in make recipes",
        1000,
        check_make_no_direct_scripts_only_atlasctl,
        fix_hint="Delegate script execution through atlasctl commands.",
    ),
    CheckDef(
        "make.no_direct_python_only_atlasctl",
        "make",
        "forbid direct python invocations in make recipes",
        1000,
        check_make_no_direct_python_only_atlasctl,
        fix_hint="Use atlasctl entrypoints instead of direct python invocations.",
    ),
    CheckDef(
        "make.no_direct_bash_ops",
        "make",
        "forbid direct `bash ops/...` invocations in make recipes",
        1000,
        check_make_no_direct_bash_ops,
        fix_hint="Route ops scripts through atlasctl make/ops commands.",
    ),
    CheckDef(
        "make.no_direct_artifact_writes",
        "make",
        "forbid direct artifact writes in make recipes",
        1000,
        check_make_no_direct_artifact_writes,
        fix_hint="Write artifacts via atlasctl reporting and command surfaces only.",
    ),
    CheckDef(
        "make.lane_reports_via_atlasctl_reporting",
        "make",
        "require lane reports to be emitted through atlasctl reporting command",
        900,
        check_make_lane_reports_via_atlasctl_reporting,
        fix_hint="Use `atlasctl report make-area-write` for lane report emission.",
    ),
    CheckDef(
        "make.ci_entrypoints_contract",
        "make",
        "validate CI workflow entrypoints contract",
        900,
        check_make_ci_entrypoints_contract,
        fix_hint="Keep workflow run commands on approved make/atlasctl entrypoints.",
    ),
    CheckDef(
        "make.ci_workflows_make_only",
        "make",
        "require CI workflows to call make entrypoints and keep atlasctl delegation in makefiles",
        900,
        check_ci_workflows_call_make_and_make_calls_atlasctl,
        fix_hint="Use `run: make <target>` in workflows and keep atlasctl delegation in makefiles.",
    ),
    CheckDef(
        "make.ci_pr_lane_fast_only",
        "make",
        "require CI PR lane suites to exclude slow checks unless explicitly allowlisted",
        900,
        check_ci_pr_lane_fast_only,
        fix_hint="Keep `local` suite fast-only, or add temporary exceptions in configs/policy/ci-pr-slow-allowlist.json.",
    ),
    CheckDef(
        "make.workflows_reference_known_suites",
        "make",
        "require workflow suite invocations to reference declared suite names",
        900,
        check_workflows_reference_known_suites,
        fix_hint="Update workflow suite names or registry/suites catalog entries so they match.",
    ),
    CheckDef(
        "make.public_targets_documented",
        "make",
        "require documented public make targets",
        900,
        check_make_public_targets_documented,
        fix_hint="Document every public make target in generated/public docs.",
    ),
    CheckDef(
        "make.target_ownership_complete",
        "make",
        "require make target ownership coverage",
        900,
        check_make_target_ownership_complete,
        fix_hint="Add missing make target ownership metadata.",
    ),
    CheckDef(
        "make.public_target_atlasctl_mapping",
        "make",
        "require every public make target to delegate to atlasctl",
        900,
        check_public_make_targets_map_to_atlasctl,
        fix_hint="Update public make target recipe bodies to call atlasctl wrappers only.",
    ),
    CheckDef(
        "make.wrapper_purity",
        "make",
        "enforce make wrapper purity for canonical makefiles",
        1000,
        check_make_wrapper_purity,
        fix_hint="Use single-line ./bin/atlasctl delegation in canonical makefiles.",
    ),
    CheckDef(
        "make.makefiles_wrappers_only_all",
        "make",
        "enforce wrapper-only invariant across canonical makefiles",
        1000,
        check_makefiles_wrappers_only_all,
        fix_hint="Keep canonical makefiles as thin wrappers that delegate via ./bin/atlasctl or approved $(MAKE) wrappers.",
    ),
    CheckDef(
        "make.wrapper_no_multiline_recipes",
        "make",
        "forbid multi-line recipes in wrapper makefiles",
        1000,
        check_make_wrapper_no_multiline_recipes,
        fix_hint="Keep wrapper targets as one-line atlasctl delegations.",
    ),
    CheckDef(
        "make.wrapper_only_calls_bin_atlasctl",
        "make",
        "require wrapper targets to call ./bin/atlasctl only",
        1000,
        check_make_wrapper_only_calls_bin_atlasctl,
        fix_hint="Use ./bin/atlasctl for wrapper recipe commands.",
    ),
    CheckDef(
        "make.wrapper_no_env_side_effects",
        "make",
        "forbid shell/env side effects inside wrapper recipe bodies",
        1000,
        check_make_wrapper_no_env_side_effects,
        fix_hint="Move environment orchestration to atlasctl commands.",
    ),
    CheckDef(
        "make.product_mk_wrapper_contract",
        "make",
        "enforce product.mk wrapper-only contract and forbid internal targets",
        1000,
        check_make_product_mk_wrapper_contract,
        fix_hint="Keep makefiles/product.mk as public atlasctl wrappers only (no internal/*, no direct python/module paths, no atlasctl run ./ops/run/*).",
    ),
    CheckDef(
        "make.no_atlasctl_run_ops_run",
        "make",
        "forbid make recipes from calling atlasctl run ./ops/run scripts",
        1000,
        check_make_no_atlasctl_run_ops_run,
        fix_hint="Use atlasctl command surfaces directly instead of `atlasctl run ./ops/run/...`.",
    ),
    CheckDef(
        "make.product_migration_complete_no_ops_run",
        "make",
        "fail if product.mk still references ops/run during migration completion",
        1000,
        check_make_product_migration_complete_no_ops_run,
        fix_hint="Replace product.mk ops/run references with atlasctl product/ops commands.",
    ),
    CheckDef(
        "make.wrapper_no_direct_cargo",
        "make",
        "forbid direct cargo calls in wrapper makefiles",
        1000,
        check_make_wrapper_no_direct_cargo,
        fix_hint="Route cargo operations through atlasctl command surfaces.",
    ),
    CheckDef(
        "make.no_python_module_invocation",
        "make",
        "forbid `python -m atlasctl.cli` in make recipes",
        1000,
        check_make_no_python_module_invocation,
        fix_hint="Use ./bin/atlasctl in make recipes.",
    ),
    CheckDef(
        "make.wrapper_no_python_m",
        "make",
        "forbid python module invocations in wrapper recipes",
        1000,
        check_make_no_python_module_invocation,
        fix_hint="Use ./bin/atlasctl entrypoints.",
    ),
    CheckDef(
        "make.wrapper_shell_is_sh",
        "make",
        "require wrapper makefiles to pin SHELL := /bin/sh",
        1000,
        check_make_wrapper_shell_is_sh,
        fix_hint="Declare SHELL := /bin/sh in each wrapper makefile.",
    ),
    CheckDef(
        "make.wrapper_phony_complete",
        "make",
        "require wrapper targets to be listed in .PHONY",
        1000,
        check_make_wrapper_phony_complete,
        fix_hint="Add each wrapper target to the makefile .PHONY list.",
    ),
    CheckDef(
        "make.no_legacy_script_aliases",
        "make",
        "forbid legacy make alias tokens (ATLAS_SCRIPTS/SCRIPTS/PY_RUN)",
        1000,
        check_make_no_legacy_script_aliases,
        fix_hint="Use ./bin/atlasctl or $(ATLASCTL) wrappers only.",
    ),
    CheckDef(
        "make.root_budget",
        "make",
        "enforce root.mk LOC/target-count budget",
        1000,
        check_make_root_budget,
        fix_hint="Reduce root.mk orchestration surface and keep only public wrappers.",
    ),
    CheckDef(
        "make.wrapper_target_budget",
        "make",
        "enforce per-makefile wrapper target count budgets",
        900,
        check_make_wrapper_target_budget,
        fix_hint="Collapse redundant wrappers and keep only canonical make targets per makefile.",
    ),
    CheckDef(
        "make.target_names_no_banned_adjectives",
        "make",
        "forbid banned adjectives in make target names",
        900,
        check_make_target_names_no_banned_adjectives,
        fix_hint="Rename make targets using neutral policy-oriented wording.",
    ),
    CheckDef(
        "make.no_duplicate_all_variants",
        "make",
        "require `*-all` targets to provide distinct full behavior",
        900,
        check_make_no_duplicate_all_variants,
        fix_hint="Keep `*-all` variants only when they add explicit full behavior flags.",
    ),
    CheckDef(
        "make.target_boundaries_enforced",
        "make",
        "enforce make target boundary contracts",
        900,
        check_make_target_boundaries_enforced,
        fix_hint="Keep target cross-area dependencies within contract boundaries.",
    ),
    CheckDef(
        "make.index_drift_contract",
        "make",
        "enforce makefiles index drift contract",
        900,
        check_make_index_drift_contract,
        fix_hint="Regenerate and commit makefiles index changes.",
    ),
    CheckDef(
        "make.no_orphan_docs_refs",
        "make",
        "forbid orphan docs references for make targets and atlasctl commands",
        900,
        check_make_no_orphan_docs_refs,
        fix_hint="Update docs index and command references so docs links are reachable.",
    ),
    CheckDef("make.scripts_refs", "make", "forbid scripts/ references in make recipes", 1000, check_make_scripts_references, fix_hint="Replace scripts/ invocations with atlasctl commands."),
    CheckDef("make.help_determinism", "make", "ensure deterministic make help output", 2000, check_make_help, fix_hint="Regenerate and normalize make help output."),
    CheckDef("make.forbidden_paths", "make", "forbid direct forbidden paths in make recipes", 1000, check_make_forbidden_paths, fix_hint="Route commands through allowed wrappers."),
    CheckDef("make.command_allowlist", "make", "enforce allowed direct recipe commands", 1500, check_make_command_allowlist, fix_hint="Use allowed command wrappers in make targets."),
    CheckDef("make.no_direct_python", "make", "forbid direct python script execution in make recipes", 1000, check_make_no_direct_python_script_invocations, fix_hint="Replace `python3 path/to/script.py` with atlasctl commands."),
    CheckDef("make.no_direct_script_exec_drift", "make", "drift check for direct script execution in make recipes", 1000, check_make_no_direct_script_exec_drift, fix_hint="Replace direct script execution with atlasctl command wrappers."),
    CheckDef("make.no_bypass_atlasctl", "make", "forbid make recipes bypassing atlasctl without explicit allowlist", 1000, check_make_no_bypass_atlasctl_without_allowlist, fix_hint="Route recipe logic through atlasctl, or add a documented delegation exception."),
    CheckDef("policies.bypass_no_new_files", "policies", "forbid unregistered new bypass files under configs/policy", 800, check_policies_bypass_no_new_files, fix_hint="Register or remove newly introduced bypass files.", tags=("repo",)),
    CheckDef("policies.bypass_all_entries_have_owner_issue_expiry", "policies", "require bypass entries to include owner, issue id, and expiry metadata", 800, check_policies_bypass_all_entries_have_owner_issue_expiry, fix_hint="Add owner/issue/expiry metadata to bypass entries.", tags=("repo",)),
    CheckDef("policies.bypass_expiry_not_past", "policies", "forbid expired bypass entries", 800, check_policies_bypass_expiry_not_past, fix_hint="Remove or renew expired bypass entries.", tags=("repo",)),
    CheckDef("policies.bypass_expiry_max_horizon", "policies", "enforce bypass expiry horizon limit", 800, check_policies_bypass_expiry_max_horizon, fix_hint="Keep expiry within max horizon or add explicit approval.", tags=("repo",)),
    CheckDef("policies.bypass_no_blank_justifications", "policies", "forbid blank bypass justifications", 800, check_policies_bypass_no_blank_justifications, fix_hint="Add concise non-empty justifications for bypass entries.", tags=("repo",)),
    CheckDef("policies.bypass_issue_id_format", "policies", "enforce bypass issue id format", 800, check_policies_bypass_issue_id_format, fix_hint="Use ISSUE-<TOKEN> issue id format.", tags=("repo",)),
    CheckDef("policies.bypass_owner_in_owners_registry", "policies", "ensure bypass owners resolve to owner registry ids", 800, check_policies_bypass_owner_in_owners_registry, fix_hint="Use owner ids in configs/meta/owners.json or configured aliases.", tags=("repo",)),
    CheckDef("policies.bypass_removal_plan_required", "policies", "require removal plans for bypass entries", 800, check_policies_bypass_removal_plan_required, fix_hint="Add removal_plan for each bypass entry.", tags=("repo",)),
    CheckDef("policies.bypass_scope_valid", "policies", "enforce valid bypass scope", 800, check_policies_bypass_scope_valid, fix_hint="Use only documented bypass scopes.", tags=("repo",)),
    CheckDef("policies.bypass_policy_name_known", "policies", "require bypass policy/rule names to be known", 800, check_policies_bypass_policy_name_known, fix_hint="Use declared policy or rule names.", tags=("repo",)),
    CheckDef("policies.bypass_schema_valid", "policies", "validate bypass JSON sources against schemas", 800, check_policies_bypass_schema_valid, fix_hint="Fix malformed bypass JSON or schema drift.", tags=("repo",)),
    CheckDef("policies.bypass_inventory_present", "policies", "require bypass inventory registry and sources to exist", 800, check_policies_bypass_inventory_present, fix_hint="Create bypass inventory SSOT registry/types and keep it in sync.", tags=("repo",)),
    CheckDef("policies.bypass_inventory_schema_valid", "policies", "validate bypass inventory sources against schemas", 800, check_policies_bypass_inventory_schema_valid, fix_hint="Fix malformed bypass JSON/TOML inventory sources or schema drift.", tags=("repo",)),
    CheckDef("policies.bypass_inventory_deterministic", "policies", "require deterministic bypass inventory output ordering", 800, check_policies_bypass_inventory_deterministic, fix_hint="Keep bypass inventory files and entries stably ordered.", tags=("repo",)),
    CheckDef("policies.bypass_has_owner", "policies", "require bypass entries to declare owner", 800, check_policies_bypass_has_owner, fix_hint="Add valid owner to each bypass entry.", tags=("repo",)),
    CheckDef("policies.bypass_has_expiry", "policies", "require bypass entries to declare expiry", 800, check_policies_bypass_has_expiry, fix_hint="Add non-expired expiry to each bypass entry.", tags=("repo",)),
    CheckDef("policies.bypass_has_reason", "policies", "require specific bypass reasons and forbid vague `because` entries", 800, check_policies_bypass_has_reason, fix_hint="Add specific justifications and avoid vague `because` reasons.", tags=("repo",)),
    CheckDef("policies.bypass_has_ticket_or_doc_ref", "policies", "require bypass entries to link issue id or local doc reference", 800, check_policies_bypass_has_ticket_or_doc_ref, fix_hint="Add issue_id or reference a local docs/ADR policy document.", tags=("repo",)),
    CheckDef("policies.bypass_budget", "policies", "enforce ratcheted bypass entry count budget", 800, check_policies_bypass_budget, fix_hint="Reduce bypass count or update budget intentionally.", tags=("repo",)),
    CheckDef("policies.bypass_budget_trend", "policies", "enforce bypass budget trend gate", 800, check_policies_bypass_budget_trend, fix_hint="Reduce bypass budget usage or update budget intentionally with approval.", tags=("repo",)),
    CheckDef("policies.bypass_readme_complete", "policies", "require configs/policy README to list every bypass source file", 800, check_policies_bypass_readme_complete, fix_hint="Update configs/policy/README.md with complete bypass source list.", tags=("repo",)),
    CheckDef("policies.bypass_readme_sorted", "policies", "require bypass source list in configs/policy README to be sorted", 800, check_policies_bypass_readme_sorted, fix_hint="Sort bypass file entries in configs/policy/README.md.", tags=("repo",)),
    CheckDef("policies.shell_network_fetch_allowlist_inline_meta", "policies", "require inline owner/why comments for network-fetch shell allowlist entries", 800, check_policies_shell_network_fetch_allowlist_inline_meta, fix_hint="Annotate every allowlist line with `# owner=<id>; why=<reason>`.", tags=("repo",)),
    CheckDef("policies.shell_probes_allowlist_inline_meta", "policies", "require inline owner/why comments for shell probe allowlist entries", 800, check_policies_shell_probes_allowlist_inline_meta, fix_hint="Annotate every allowlist line with `# owner=<id>; why=<reason>`.", tags=("repo",)),
    CheckDef("policies.adjectives_repo_clean", "policies", "forbid forbidden adjectives across tracked files", 800, check_policies_adjectives_repo_clean, fix_hint="Replace forbidden adjectives with neutral wording.", tags=("repo",)),
    CheckDef("policies.adjective_allowlist_budget", "policies", "enforce adjective allowlist ratchet budget", 800, check_policies_adjective_allowlist_budget, fix_hint="Keep adjective allowlist empty or within ratchet budget.", tags=("repo",)),
    CheckDef("policies.bypass_files_scoped", "policies", "ensure bypass files are scoped under configs/policy", 800, check_policies_bypass_files_scoped, fix_hint="Move bypass files into configs/policy or remove them.", tags=("repo",)),
    CheckDef("policies.no_inline_bypass_entries", "policies", "forbid inline bypass metadata in source code", 800, check_policies_no_inline_bypass_entries, fix_hint="Move bypass metadata to configs/policy files.", tags=("repo",)),
    CheckDef("policies.tests_bypass_dependency_marked", "policies", "require explicit marker when tests depend on bypass files", 800, check_policies_tests_bypass_dependency_marked, fix_hint="Add `# BYPASS_TEST_OK` marker to tests that intentionally read bypass files.", tags=("repo",)),
    CheckDef("policies.bypass_usage_heatmap", "policies", "emit bypass usage heatmap report", 800, check_policies_bypass_usage_heatmap, fix_hint="Review bypass usage report and reduce hotspots.", tags=("repo",)),
    CheckDef("policies.bypass_removal_milestones_defined", "policies", "require bypass removal milestone SSOT file", 800, check_policies_bypass_removal_milestones_defined, fix_hint="Define removal milestones in configs/policy/bypass-removal-milestones.json.", tags=("repo",)),
    CheckDef("policies.bypass_count_nonincreasing", "policies", "enforce migration gate: bypass count must not increase", 800, check_policies_bypass_count_nonincreasing, fix_hint="Reduce bypass count or intentionally update baseline in one reviewed change.", tags=("repo",)),
    CheckDef("policies.bypass_count_nonincreasing_hard", "policies", "hard fail invariant: bypass count must not increase", 800, check_policies_bypass_count_nonincreasing, fix_hint="Reduce bypass count or intentionally update baseline in one reviewed change.", tags=("repo", "required")),
    CheckDef("policies.bypass_new_entries_forbidden", "policies", "forbid new bypass entries unless explicitly approved", 800, check_policies_bypass_new_entries_forbidden, fix_hint="Reduce bypass count or add temporary approvals in configs/policy/bypass-new-entry-approvals.json.", tags=("repo",)),
    CheckDef("policies.bypass_hard_gate", "policies", "fail when bypass hard gate is enabled and any bypass entries exist", 800, check_policies_bypass_hard_gate, fix_hint="Remove all bypass entries or disable hard gate until ready to enforce zero-bypass milestone.", tags=("repo", "policies", "required")),
    CheckDef("policies.bypass_mainline_strict_mode", "policies", "optional strict mode: fail on any bypass entry when enabled", 800, check_policies_bypass_mainline_strict_mode, fix_hint="Enable only after bypass count reaches zero or acceptable strict milestone.", tags=("repo", "policies")),
    CheckDef("policies.bypass_has_test_coverage", "policies", "require bypass entries to have declared validating test coverage", 800, check_policies_bypass_has_test_coverage, fix_hint="Map bypass sources/entries to validating tests in configs/policy/bypass-test-coverage.json.", tags=("repo",)),
    CheckDef("policies.bypass_ids_unique", "policies", "require unique bypass IDs across policy files", 800, check_policies_bypass_ids_unique, fix_hint="Deduplicate bypass IDs/keys across configs/policy.", tags=("repo",)),
    CheckDef("policies.bypass_entry_paths_exist", "policies", "fail if bypass entry points to missing file/path", 800, check_policies_bypass_entry_paths_exist, fix_hint="Fix or remove stale bypass references to missing paths.", tags=("repo",)),
    CheckDef("policies.bypass_entry_matches_nothing", "policies", "fail if wildcard bypass entry matches nothing", 800, check_policies_bypass_entry_matches_nothing, fix_hint="Tighten or remove stale wildcard bypass entries.", tags=("repo",)),
    CheckDef("policies.bypass_entry_matches_too_broad", "policies", "fail if wildcard bypass entry matches too broadly", 800, check_policies_bypass_entry_matches_too_broad, fix_hint="Narrow wildcard bypass scope to intended paths only.", tags=("repo",)),
    CheckDef("policies.dependency_exceptions_remove_by_issue", "policies", "require dependency exceptions to include remove_by and issue_id", 800, check_policies_dependency_exceptions_remove_by_issue, fix_hint="Add `remove_by` and valid `issue_id` to dependency exceptions.", tags=("repo",)),
    CheckDef("policies.budget_relaxation_requires_approval", "policies", "require explicit approval for any budget relaxation", 800, check_policies_budget_relaxation_requires_approval, fix_hint="Set budget-loosening-approval.json with approved=true and approval_id when relaxations exist.", tags=("repo",)),
    CheckDef("policies.budget_approval_time_bounded", "policies", "require budget loosening approvals to be time bounded", 800, check_policies_budget_approval_time_bounded, fix_hint="Set valid non-expired bounded expiry in budget-loosening-approval.json.", tags=("repo",)),
)
