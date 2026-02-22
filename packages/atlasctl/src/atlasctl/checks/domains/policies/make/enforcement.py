from __future__ import annotations
import json
import re
import runpy
import datetime as dt
from pathlib import Path
from typing import Iterable

from .....commands.dev.make.public_targets import public_names
from ....repo.native import (
    check_make_no_direct_python_script_invocations,
    check_make_scripts_references,
)
_MAKE_RECIPE_RE = re.compile(r"^\t(?P<body>.*)$")
_MAKE_TARGET_RE = re.compile(r"^(?P<target>[A-Za-z0-9_./-]+):(?:\s|$)")
_SCRIPT_PATH_RE = re.compile(r"(^|\s)(?:\./)?(?:ops|scripts|packages/atlasctl/src/atlasctl)/[^\s]+\.(?:sh|py)(?:\s|$)")
_BASH_OPS_RE = re.compile(r"(?:^|\s)(?:bash|sh)\s+(?:\./)?ops/[^\s]+")
_WRITE_RE = re.compile(r"(?:^|\s)(?:cp\s+[^\n]*\s+|mv\s+[^\n]*\s+|cat\s+>\s*|tee\s+|mkdir\s+-p\s+|touch\s+|>\s*|>>\s*)([^\s\"';]+)")
_ATLASCTL_MODULE_RE = re.compile(r"\bpython3?\s+-m\s+atlasctl\.cli\b")
_ATLASCTL_SUITE_RUN_RE = re.compile(r"\batlasctl\s+suite\s+run\s+([A-Za-z0-9_.-]+)\b")
_LEGACY_MAKE_ALIAS_RE = re.compile(r"\$\((ATLAS_SCRIPTS|SCRIPTS|PY_RUN)\)|\b(ATLAS_SCRIPTS|SCRIPTS|PY_RUN)\b")
_WRAPPER_FILES = (
    "makefiles/dev.mk",
    "makefiles/docs.mk",
    "makefiles/ops.mk",
    "makefiles/ci.mk",
    "makefiles/policies.mk",
    "makefiles/product.mk",
    "makefiles/layout.mk",
    "makefiles/env.mk",
    "makefiles/root.mk",
)
_ROOT_MK_MAX_LOC = 900
_ROOT_MK_MAX_TARGETS = 220
_ISSUE_ID_RE = re.compile(r"^ISSUE-[A-Z0-9-]+$")
_BYPASS_SOURCES = (("configs/policy/policy-relaxations.json", "json", True), ("configs/policy/effect-boundary-exceptions.json", "json", False), ("configs/policy/dead-modules-allowlist.json", "json", False), ("configs/policy/dependency-exceptions.json", "json", True), ("configs/policy/layer-relaxations.json", "json", True), ("configs/policy/budget-relaxations.json", "json", True), ("configs/policy/ops-lint-relaxations.json", "json", True), ("configs/policy/ops-smoke-budget-relaxations.json", "json", True), ("configs/policy/pin-relaxations.json", "json", True), ("configs/policy/check-filename-allowlist.json", "json", False), ("configs/policy/layer-live-diff-allowlist.json", "json", False), ("configs/policy/slow-checks-ratchet.json", "json", False), ("configs/policy/forbidden-adjectives-allowlist.txt", "txt", False), ("configs/policy/shell-network-fetch-allowlist.txt", "txt", False), ("configs/policy/shell-probes-allowlist.txt", "txt", False))
_BYPASS_SCHEMAS = {"configs/policy/policy-relaxations.json": "configs/_schemas/policy-relaxations.schema.json", "configs/policy/layer-relaxations.json": "configs/_schemas/layer-relaxations.schema.json", "configs/policy/ops-lint-relaxations.json": "configs/_schemas/ops-lint-relaxations.schema.json", "configs/policy/layer-live-diff-allowlist.json": "configs/_schemas/layer-live-diff-allowlist.schema.json", "configs/policy/effect-boundary-exceptions.json": "configs/policy/bypass.schema.json", "configs/policy/dead-modules-allowlist.json": "configs/policy/bypass.schema.json", "configs/policy/dependency-exceptions.json": "configs/policy/bypass.schema.json", "configs/policy/budget-relaxations.json": "configs/policy/bypass.schema.json", "configs/policy/ops-smoke-budget-relaxations.json": "configs/policy/bypass.schema.json", "configs/policy/pin-relaxations.json": "configs/policy/bypass.schema.json", "configs/policy/check-filename-allowlist.json": "configs/policy/bypass.schema.json", "configs/policy/slow-checks-ratchet.json": "configs/policy/bypass.schema.json"}
_BYPASS_SCOPES = {"repo", "crate", "module", "file", "path", "docs", "ops", "make", "workflow", "policy"}
def _iter_make_recipe_lines(repo_root: Path) -> list[tuple[str, int, str]]:
    rows: list[tuple[str, int, str]] = []
    files = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    for path in files:
        rel = path.relative_to(repo_root).as_posix()
        if rel == "makefiles/_macros.mk":
            # Macro file contains shell snippets inside `define`; these are not executable targets.
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            match = _MAKE_RECIPE_RE.match(line)
            if not match:
                continue
            body = match.group("body").strip()
            if not body or body.startswith("#"):
                continue
            rows.append((rel, lineno, body))
    return rows


def _iter_make_targets(repo_root: Path, rel_path: str) -> list[tuple[str, list[tuple[int, str]]]]:
    path = repo_root / rel_path
    if not path.exists():
        return []
    targets: list[tuple[str, list[tuple[int, str]]]] = []
    current_target = ""
    current_lines: list[tuple[int, str]] = []
    for lineno, raw in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
        target_match = _MAKE_TARGET_RE.match(raw)
        if target_match and not raw.startswith("."):
            if current_target:
                targets.append((current_target, current_lines))
            current_target = target_match.group("target")
            current_lines = []
            continue
        recipe_match = _MAKE_RECIPE_RE.match(raw)
        if recipe_match and current_target:
            body = recipe_match.group("body").strip()
            if body:
                current_lines.append((lineno, body))
    if current_target:
        targets.append((current_target, current_lines))
    return targets


def _load_exceptions(repo_root: Path, kind: str) -> set[str]:
    cfg_path = repo_root / "configs/make/delegation-exceptions.json"
    if not cfg_path.exists():
        return set()
    payload = json.loads(cfg_path.read_text(encoding="utf-8"))
    rows = payload.get(kind, [])
    if not isinstance(rows, list):
        return set()
    return {str(item).strip() for item in rows if str(item).strip()}


def check_make_no_direct_scripts_only_atlasctl(repo_root: Path) -> tuple[int, list[str]]:
    exceptions = _load_exceptions(repo_root, "direct_scripts")
    errors: list[str] = []
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if "atlasctl" in body or "$(ATLAS_SCRIPTS)" in body:
            continue
        if _SCRIPT_PATH_RE.search(body) is None:
            continue
        msg = f"{rel}:{lineno}: direct script path invocation is forbidden in make recipes"
        if msg in exceptions:
            continue
        errors.append(msg)
    return (0 if not errors else 1), sorted(errors)


def check_make_no_direct_python_only_atlasctl(repo_root: Path) -> tuple[int, list[str]]:
    code, errors = check_make_no_direct_python_script_invocations(repo_root)
    if code == 0:
        return 0, []
    exceptions = _load_exceptions(repo_root, "direct_python")
    filtered = [err for err in errors if err not in exceptions]
    return (0 if not filtered else 1), sorted(filtered)


def check_make_no_direct_bash_ops(repo_root: Path) -> tuple[int, list[str]]:
    exceptions = _load_exceptions(repo_root, "direct_bash_ops")
    errors: list[str] = []
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if _BASH_OPS_RE.search(body) is None:
            continue
        msg = f"{rel}:{lineno}: direct `bash ops/...` invocation is forbidden in make recipes"
        if msg in exceptions:
            continue
        errors.append(msg)
    return (0 if not errors else 1), sorted(errors)


def check_make_no_direct_artifact_writes(repo_root: Path) -> tuple[int, list[str]]:
    exceptions = _load_exceptions(repo_root, "direct_artifact_writes")
    errors: list[str] = []
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if "atlasctl" in body or "$(ATLAS_SCRIPTS)" in body:
            continue
        match = _WRITE_RE.search(body)
        if not match:
            continue
        target = match.group(1)
        if not target.startswith("artifacts/"):
            continue
        msg = f"{rel}:{lineno}: direct artifact writes are forbidden in make recipes (`{target}`)"
        if msg in exceptions:
            continue
        errors.append(msg)
    return (0 if not errors else 1), sorted(errors)


def _run_script(repo_root: Path, script: str) -> tuple[int, list[str]]:
    script_path = repo_root / script
    if not script_path.exists():
        return 1, [f"missing script: {script}"]
    try:
        runpy.run_path(str(script_path), run_name="__main__")
        return 0, []
    except SystemExit as exc:
        code = int(exc.code) if isinstance(exc.code, int) else 1
        if code == 0:
            return 0, []
        return 1, [f"script exited non-zero: {script} (code={code})"]
    except Exception as exc:
        return 1, [f"script execution failed: {script}: {exc}"]


def check_make_ci_entrypoints_contract(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/layout/workflows/check_ci_entrypoints.py")


def check_make_public_targets_documented(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(
        repo_root,
        "packages/atlasctl/src/atlasctl/checks/layout/domains/public_surface/check_public_targets_documented.py",
    )


def check_make_target_ownership_complete(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/domains/policies/make/check_make_target_ownership.py")


def check_make_target_boundaries_enforced(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/domains/policies/make/check_makefile_target_boundaries.py")


def check_make_index_drift_contract(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/layout/makefiles/index/check_makefiles_index_drift.py")


def check_make_no_orphan_docs_refs(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/layout/docs/check_no_orphan_docs_refs.py")


def check_make_no_direct_scripts_legacy(repo_root: Path) -> tuple[int, list[str]]:
    code, errors = check_make_scripts_references(repo_root)
    if code == 0:
        return 0, []
    exceptions = _load_exceptions(repo_root, "legacy_scripts_refs")
    filtered = [err for err in errors if err not in exceptions]
    return (0 if not filtered else 1), sorted(filtered)


def check_make_lane_reports_via_atlasctl_reporting(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    files = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    for path in files:
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            if "lane_report.sh" in line:
                errors.append(f"{rel}:{lineno}: lane reporting must use `atlasctl report make-area-write`")
    return (0 if not errors else 1), errors


def check_make_no_direct_script_exec_drift(repo_root: Path) -> tuple[int, list[str]]:
    # Explicit drift check alias for direct script invocation prohibition.
    return check_make_no_direct_scripts_only_atlasctl(repo_root)


def check_make_no_bypass_atlasctl_without_allowlist(repo_root: Path) -> tuple[int, list[str]]:
    exceptions = _load_exceptions(repo_root, "bypass_atlasctl")
    errors: list[str] = []
    wrapper_purity_files = {"makefiles/dev.mk", "makefiles/ci.mk", "makefiles/cargo.mk"}
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if rel not in wrapper_purity_files:
            continue
        if "atlasctl" in body or "$(ATLAS_SCRIPTS)" in body or "$(SCRIPTS)" in body or "$(MAKE)" in body:
            continue
        if body.startswith("@echo ") or body.startswith("echo "):
            continue
        msg = f"{rel}:{lineno}: make recipe bypasses atlasctl wrapper"
        if msg in exceptions:
            continue
        errors.append(msg)
    return (0 if not errors else 1), sorted(errors)


def check_make_wrapper_purity(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel in _WRAPPER_FILES:
        if rel == "makefiles/root.mk":
            continue
        for target, recipe_lines in _iter_make_targets(repo_root, rel):
            if not recipe_lines:
                continue
            if target.startswith("internal/"):
                continue
            if len(recipe_lines) != 1:
                errors.append(f"{rel}:{target}: wrapper target must have exactly one recipe line")
                continue
            lineno, body = recipe_lines[0]
            if target == "help":
                continue
            if body.startswith("@./bin/atlasctl ") or body.startswith("./bin/atlasctl "):
                continue
            if "./bin/atlasctl " in body and body.count("./bin/atlasctl") == 1:
                continue
            if body.startswith("@$(MAKE) ") or body.startswith("$(MAKE) "):
                continue
            if "$(MAKE)" in body:
                continue
            errors.append(f"{rel}:{lineno}: wrapper recipe must delegate via ./bin/atlasctl")
    return (0 if not errors else 1), sorted(errors)


def check_make_no_python_module_invocation(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if _ATLASCTL_MODULE_RE.search(body):
            errors.append(f"{rel}:{lineno}: forbidden python module invocation in make recipe (`python -m atlasctl.cli`)")
    return (0 if not errors else 1), sorted(errors)


def check_make_root_budget(repo_root: Path) -> tuple[int, list[str]]:
    root_mk = repo_root / "makefiles/root.mk"
    if not root_mk.exists():
        return 1, ["missing makefiles/root.mk"]
    text = root_mk.read_text(encoding="utf-8", errors="ignore")
    loc = len(text.splitlines())
    targets = [line for line in text.splitlines() if _MAKE_TARGET_RE.match(line) and not line.startswith(".")]
    errors: list[str] = []
    if loc > _ROOT_MK_MAX_LOC:
        errors.append(f"makefiles/root.mk exceeds LOC budget ({loc} > {_ROOT_MK_MAX_LOC})")
    if len(targets) > _ROOT_MK_MAX_TARGETS:
        errors.append(f"makefiles/root.mk exceeds target-count budget ({len(targets)} > {_ROOT_MK_MAX_TARGETS})")
    return (0 if not errors else 1), errors




def check_ci_workflows_call_make_and_make_calls_atlasctl(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    workflows = sorted((repo_root / ".github" / "workflows").glob("*.yml"))
    for wf in workflows:
        text = wf.read_text(encoding="utf-8", errors="ignore")
        run_lines = [line.strip() for line in text.splitlines() if line.strip().startswith("run:")]
        if not run_lines:
            continue
        if any(re.search(r"\bpython3?\s+-m\s+atlasctl(\.cli)?\b", line) for line in run_lines):
            errors.append(
                f"{wf.relative_to(repo_root).as_posix()}: workflow must not invoke atlasctl via `python -m`; use `./bin/atlasctl`"
            )
        if any(re.search(r"\bcargo\s+(fmt|test|clippy|check)\b", line) for line in run_lines):
            errors.append(
                f"{wf.relative_to(repo_root).as_posix()}: workflow must not run raw cargo fmt/test/clippy/check; use make/atlasctl wrappers"
            )
    return (0 if not errors else 1), sorted(errors)


def _workflow_texts(repo_root: Path) -> Iterable[tuple[str, str]]:
    for wf in sorted((repo_root / ".github" / "workflows").glob("*.yml")):
        yield wf.relative_to(repo_root).as_posix(), wf.read_text(encoding="utf-8", errors="ignore")


def check_workflows_reference_known_suites(repo_root: Path) -> tuple[int, list[str]]:
    from .....registry.suites import suite_manifest_specs

    known = {spec.name for spec in suite_manifest_specs()}
    errors: list[str] = []
    for rel, text in _workflow_texts(repo_root):
        for lineno, line in enumerate(text.splitlines(), start=1):
            m = _ATLASCTL_SUITE_RUN_RE.search(line)
            if not m:
                continue
            suite_name = m.group(1)
            if suite_name not in known:
                errors.append(f"{rel}:{lineno}: unknown suite in workflow: atlasctl suite run {suite_name}")
    return (0 if not errors else 1), sorted(errors)


def check_ci_pr_lane_fast_only(repo_root: Path) -> tuple[int, list[str]]:
    from .....checks.registry import get_check
    from .....registry.suites import resolve_check_ids, suite_manifest_specs

    allowlist_path = repo_root / "configs/policy/ci-pr-slow-allowlist.json"
    allowlist: set[str] = set()
    if allowlist_path.exists():
        payload = json.loads(allowlist_path.read_text(encoding="utf-8"))
        allowlist = {str(item) for item in payload.get("allowlist", [])}

    spec_by_name = {spec.name: spec for spec in suite_manifest_specs()}
    pr_spec = spec_by_name.get("local")
    if pr_spec is None:
        return 1, ["missing `local` suite spec; required for CI PR fast lane"]
    selected = resolve_check_ids(pr_spec)
    slow_checks = [cid for cid in selected if (check := get_check(cid)) is not None and check.slow and cid not in allowlist]
    if slow_checks:
        return 1, [f"CI PR lane includes slow checks without allowlist: {', '.join(sorted(slow_checks))}"]
    return 0, []


def check_public_make_targets_map_to_atlasctl(repo_root: Path) -> tuple[int, list[str]]:
    files = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    texts = {path.relative_to(repo_root).as_posix(): path.read_text(encoding="utf-8", errors="ignore") for path in files}
    errors: list[str] = []
    for target in public_names():
        pattern = re.compile(rf"(?m)^{re.escape(target)}:\s.*?(?:\n(?:\t.*\n?)*)")
        match_rel: str | None = None
        match_block: str | None = None
        for rel, text in texts.items():
            match = pattern.search(text)
            if match is None:
                continue
            match_rel = rel
            match_block = match.group(0)
            break
        if match_block is None:
            errors.append(f"public target missing from make surface: {target}")
            continue
        block = match_block
        if "atlasctl" not in block and "$(ATLAS_SCRIPTS)" not in block and "$(SCRIPTS)" not in block and "$(MAKE)" not in block:
            errors.append(f"public target must delegate to atlasctl wrappers: {target} ({match_rel})")
    return (0 if not errors else 1), sorted(errors)


def check_make_no_legacy_script_aliases(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    files = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    for path in files:
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if _LEGACY_MAKE_ALIAS_RE.search(line) is None:
                continue
            errors.append(f"{rel}:{lineno}: legacy make alias token is forbidden (`ATLAS_SCRIPTS`, `SCRIPTS`, `PY_RUN`)")
    return (0 if not errors else 1), sorted(errors)


def _load_bypass_entries(repo_root: Path) -> list[dict[str, object]]:
    overrides_path = repo_root / "configs/policy/bypass-entry-overrides.json"
    overrides_payload = json.loads(overrides_path.read_text(encoding="utf-8")) if overrides_path.exists() else {}
    overrides_rows = overrides_payload.get("overrides", []) if isinstance(overrides_payload, dict) else []
    overrides = {
        str(row.get("key", "")).strip(): row
        for row in overrides_rows
        if isinstance(row, dict) and str(row.get("key", "")).strip()
    }
    entries: list[dict[str, object]] = []
    for rel_path, kind, requires_metadata in _BYPASS_SOURCES:
        path = repo_root / rel_path
        if not path.exists():
            continue
        if kind == "txt":
            for idx, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                value = line.strip()
                if not value or value.startswith("#"):
                    continue
                entries.append({"source": rel_path, "key": f"{rel_path}#{idx}", "requires_metadata": False})
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        if rel_path.endswith("effect-boundary-exceptions.json"):
            rules = payload.get("rules", {}) if isinstance(payload, dict) else {}
            if isinstance(rules, dict):
                for rule, values in rules.items():
                    if not isinstance(values, list):
                        continue
                    for item in values:
                        if not isinstance(item, dict):
                            continue
                        item_key = str(item.get("path", "")).strip() or f"{rule}#{len(entries)+1}"
                        row: dict[str, object] = {
                            "source": rel_path,
                            "key": item_key,
                            "policy_name": str(rule),
                            "scope": "",
                            "owner": "",
                            "issue_id": "",
                            "expiry": "",
                            "justification": str(item.get("reason", "")).strip(),
                            "removal_plan": "",
                            "requires_metadata": requires_metadata,
                        }
                        row.update({k: v for k, v in overrides.get(f"{rel_path}:{item_key}", {}).items() if k != "key"})
                        entries.append(row)
            continue
        if not isinstance(payload, dict):
            continue
        for list_key in ("exceptions", "relaxations", "allow", "allowlist", "undeclared_import_allowlist", "optional_dependency_usage_allowlist", "internal_third_party_allowlist"):
            values = payload.get(list_key)
            if not isinstance(values, list):
                continue
            for item in values:
                if isinstance(item, str):
                    item = {"path": item}
                if not isinstance(item, dict):
                    continue
                item_key = str(item.get("id") or item.get("check_id") or item.get("path") or f"{list_key}#{len(entries)+1}").strip()
                row = {
                    "source": rel_path,
                    "key": item_key,
                    "policy_name": str(item.get("policy") or item.get("rule") or list_key).strip(),
                    "scope": str(item.get("scope", "")).strip(),
                    "owner": str(item.get("owner", "")).strip(),
                    "issue_id": str(item.get("issue_id") or item.get("issue") or "").strip(),
                    "expiry": str(item.get("expiry") or item.get("expires_on") or "").strip(),
                    "justification": str(item.get("justification") or item.get("reason") or "").strip(),
                    "removal_plan": str(item.get("removal_plan", "")).strip(),
                    "requires_metadata": requires_metadata,
                }
                row.update({k: v for k, v in overrides.get(f"{rel_path}:{item_key}", {}).items() if k != "key"})
                entries.append(row)
    return entries


def collect_bypass_inventory(repo_root: Path) -> dict[str, object]:
    files: list[dict[str, object]] = []
    entries = _load_bypass_entries(repo_root)
    counts: dict[str, int] = {}
    for entry in entries:
        source = str(entry.get("source", ""))
        counts[source] = counts.get(source, 0) + 1
    for rel_path, kind, _ in _BYPASS_SOURCES:
        files.append({"path": rel_path, "exists": (repo_root / rel_path).exists(), "kind": kind, "entry_count": counts.get(rel_path, 0)})
    return {"schema_version": 1, "files": files, "entry_count": len(entries), "entries": entries, "errors": []}


def render_text_report(payload: dict[str, object]) -> str:
    files = payload.get("files", []) if isinstance(payload.get("files", []), list) else []
    entry_count = int(payload.get("entry_count", 0))
    lines = ["Policy Bypass Inventory", f"files: {len(files)}", f"entries: {entry_count}", "", "Files:"]
    for row in files:
        if not isinstance(row, dict):
            continue
        lines.append(f"- {row.get('path')}: exists={row.get('exists')} kind={row.get('kind')} entries={row.get('entry_count')}")
    return "\n".join(lines)


def _bypass_errors(repo_root: Path) -> dict[str, list[str]]:
    entries = _load_bypass_entries(repo_root)
    by_check: dict[str, list[str]] = {k: [] for k in ("meta", "expiry", "horizon", "justification", "issue", "owner", "removal", "scope", "policy", "schema", "budget", "files")}
    known_files = {row[0] for row in _BYPASS_SOURCES} | {"configs/policy/migration_exceptions.json", "configs/policy/checks-registry-transition.json", "configs/policy/check-id-migration.json", "configs/policy/forbidden-adjectives-approvals.json", "configs/policy/check_speed_approvals.json"}
    actual_files = {p.relative_to(repo_root).as_posix() for p in (repo_root / "configs/policy").glob("*") if p.is_file() and any(t in p.name for t in ("allowlist", "relax", "exceptions", "ratchet"))}
    for path in sorted(actual_files - known_files):
        by_check["files"].append(f"unexpected bypass file under configs/policy: {path}")
    owners_path = repo_root / "configs/meta/owners.json"
    owners = {str(item.get("id", "")).strip() for item in json.loads(owners_path.read_text(encoding="utf-8")).get("owners", []) if isinstance(item, dict)} if owners_path.exists() else set()
    aliases_payload = json.loads((repo_root / "configs/policy/bypass-owner-aliases.json").read_text(encoding="utf-8")) if (repo_root / "configs/policy/bypass-owner-aliases.json").exists() else {}
    aliases = aliases_payload.get("aliases", {}) if isinstance(aliases_payload, dict) else {}
    approvals_payload = json.loads((repo_root / "configs/policy/bypass-horizon-approvals.json").read_text(encoding="utf-8")) if (repo_root / "configs/policy/bypass-horizon-approvals.json").exists() else {}
    approvals = {str(row.get("key", "")).strip() for row in approvals_payload.get("approvals", []) if isinstance(row, dict)}
    policy_relax = json.loads((repo_root / "configs/policy/policy-relaxations.json").read_text(encoding="utf-8"))
    known_policy = set(policy_relax.get("exception_budgets", {}).keys())
    layer = json.loads((repo_root / "configs/policy/layer-relaxations.json").read_text(encoding="utf-8"))
    known_policy.update(str(row.get("rule", "")).strip() for row in layer.get("exceptions", []) if isinstance(row, dict))
    known_policy.update({"allow", "allowlist", "exceptions", "relaxations", "undeclared_import_allowlist", "optional_dependency_usage_allowlist", "internal_third_party_allowlist"})
    today = dt.date.today()
    for row in entries:
        if not bool(row.get("requires_metadata", False)):
            continue
        source = str(row.get("source", ""))
        key = str(row.get("key", ""))
        owner = str(row.get("owner", "")).strip()
        issue_id = str(row.get("issue_id", "")).strip()
        expiry_raw = str(row.get("expiry", "")).strip()
        scope = str(row.get("scope", "")).strip()
        policy_name = str(row.get("policy_name", "")).strip()
        if not owner or not issue_id or not expiry_raw:
            by_check["meta"].append(f"{source}:{key}: missing owner/issue/expiry")
        if not str(row.get("justification", "")).strip():
            by_check["justification"].append(f"{source}:{key}: blank justification")
        if not _ISSUE_ID_RE.match(issue_id):
            by_check["issue"].append(f"{source}:{key}: invalid issue id format `{issue_id}`")
        if str(row.get("removal_plan", "")).strip() == "":
            by_check["removal"].append(f"{source}:{key}: removal_plan is required")
        if scope not in _BYPASS_SCOPES:
            by_check["scope"].append(f"{source}:{key}: invalid scope `{scope}`")
        if not policy_name or policy_name not in known_policy:
            by_check["policy"].append(f"{source}:{key}: unknown policy/rule `{policy_name}`")
        resolved_owner = str(aliases.get(owner, owner))
        if resolved_owner not in owners:
            by_check["owner"].append(f"{source}:{key}: owner `{owner}` not in owners registry")
        try:
            expiry = dt.date.fromisoformat(expiry_raw)
        except ValueError:
            by_check["expiry"].append(f"{source}:{key}: invalid expiry `{expiry_raw}`")
            continue
        if expiry < today:
            by_check["expiry"].append(f"{source}:{key}: expired on {expiry_raw}")
        delta = (expiry - today).days
        if delta > 90 and "*" not in approvals and key not in approvals:
            by_check["horizon"].append(f"{source}:{key}: expiry horizon {delta}d exceeds 90d without approval")
    try:
        import jsonschema
        for rel_path, _, _ in _BYPASS_SOURCES:
            schema_rel = _BYPASS_SCHEMAS.get(rel_path)
            if not schema_rel:
                continue
            p = repo_root / rel_path
            if not p.exists():
                continue
            payload = json.loads(p.read_text(encoding="utf-8"))
            schema = json.loads((repo_root / schema_rel).read_text(encoding="utf-8"))
            try:
                jsonschema.validate(payload, schema)
            except jsonschema.ValidationError as exc:
                by_check["schema"].append(f"{rel_path}: schema violation ({exc.message})")
    except ModuleNotFoundError:
        by_check["schema"].append("jsonschema package is required for bypass schema validation")
    budget_path = repo_root / "configs/policy/bypass-budget.json"
    if not budget_path.exists():
        by_check["budget"].append("configs/policy/bypass-budget.json missing")
    else:
        payload = json.loads(budget_path.read_text(encoding="utf-8"))
        max_entries = int(payload.get("max_entries", 0))
        if max_entries <= 0:
            by_check["budget"].append("configs/policy/bypass-budget.json: max_entries must be > 0")
        elif len(entries) > max_entries:
            by_check["budget"].append(f"bypass budget exceeded: entries={len(entries)} max_entries={max_entries}")
    return by_check


def _res(errors: list[str]) -> tuple[int, list[str]]:
    uniq = sorted(set(errors))
    return (0 if not uniq else 1), uniq
def _bypass_readme_errors(repo_root: Path) -> dict[str, list[str]]:
    path = repo_root / "configs/policy/README.md"
    if not path.exists():
        return {"complete": ["configs/policy/README.md missing"], "sorted": ["configs/policy/README.md missing"]}
    text = path.read_text(encoding="utf-8")
    refs = sorted(set(re.findall(r"`(configs/policy/[^`]+)`", text)))
    expected = sorted(row[0] for row in _BYPASS_SOURCES)
    missing = sorted(item for item in expected if item not in refs); extra = sorted(item for item in refs if item not in expected)
    complete: list[str] = [*(f"README missing bypass file entry: {item}" for item in missing), *(f"README lists non-bypass file entry: {item}" for item in extra)]
    order = [item for item in refs if item in expected]
    sorted_errors = [] if order == sorted(order) else ["configs/policy/README.md bypass entries must be sorted"]
    return {"complete": complete, "sorted": sorted_errors}

def check_policies_bypass_no_new_files(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["files"])
def check_policies_bypass_all_entries_have_owner_issue_expiry(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["meta"])
def check_policies_bypass_expiry_not_past(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["expiry"])
def check_policies_bypass_expiry_max_horizon(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["horizon"])
def check_policies_bypass_no_blank_justifications(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["justification"])
def check_policies_bypass_issue_id_format(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["issue"])
def check_policies_bypass_owner_in_owners_registry(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["owner"])
def check_policies_bypass_removal_plan_required(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["removal"])
def check_policies_bypass_scope_valid(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["scope"])
def check_policies_bypass_policy_name_known(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["policy"])
def check_policies_bypass_schema_valid(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["schema"])
def check_policies_bypass_budget(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["budget"])
def check_policies_bypass_readme_complete(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_readme_errors(repo_root)["complete"])
def check_policies_bypass_readme_sorted(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_readme_errors(repo_root)["sorted"])
