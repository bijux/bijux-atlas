from __future__ import annotations
import json
import os
import re
import runpy
import datetime as dt
import fnmatch
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
    "makefiles/atlasctl.mk",
    "makefiles/dev.mk",
    "makefiles/docs.mk",
    "makefiles/ops.mk",
    "makefiles/ci.mk",
    "makefiles/policies.mk",
    "makefiles/product.mk",
    "makefiles/env.mk",
    "makefiles/verification.mk",
    "makefiles/root.mk",
)
_ROOT_MK_MAX_LOC = 900
_ROOT_MK_MAX_TARGETS = 220
_ISSUE_ID_RE = re.compile(r"^ISSUE-[A-Z0-9-]+$")
_BYPASS_SOURCES = (("configs/policy/policy-relaxations.json", "json", True), ("configs/policy/effect-boundary-exceptions.json", "json", False), ("configs/policy/dead-modules-allowlist.json", "json", False), ("configs/policy/dependency-exceptions.json", "json", True), ("configs/policy/layer-relaxations.json", "json", True), ("configs/policy/budget-relaxations.json", "json", True), ("configs/policy/ops-lint-relaxations.json", "json", True), ("configs/policy/ops-smoke-budget-relaxations.json", "json", True), ("configs/policy/pin-relaxations.json", "json", True), ("configs/policy/check-filename-allowlist.json", "json", False), ("configs/policy/layer-live-diff-allowlist.json", "json", False), ("configs/policy/slow-checks-ratchet.json", "json", False), ("configs/policy/forbidden-adjectives-allowlist.txt", "txt", False), ("configs/policy/shell-network-fetch-allowlist.txt", "txt", False), ("configs/policy/shell-probes-allowlist.txt", "txt", False))
_BYPASS_SCHEMAS = {"configs/policy/policy-relaxations.json": "configs/_schemas/policy-relaxations.schema.json", "configs/policy/layer-relaxations.json": "configs/_schemas/layer-relaxations.schema.json", "configs/policy/ops-lint-relaxations.json": "configs/_schemas/ops-lint-relaxations.schema.json", "configs/policy/layer-live-diff-allowlist.json": "configs/_schemas/layer-live-diff-allowlist.schema.json", "configs/policy/effect-boundary-exceptions.json": "configs/policy/bypass.schema.json", "configs/policy/dead-modules-allowlist.json": "configs/policy/bypass.schema.json", "configs/policy/dependency-exceptions.json": "configs/policy/bypass.schema.json", "configs/policy/budget-relaxations.json": "configs/policy/bypass.schema.json", "configs/policy/ops-smoke-budget-relaxations.json": "configs/policy/bypass.schema.json", "configs/policy/pin-relaxations.json": "configs/policy/bypass.schema.json", "configs/policy/check-filename-allowlist.json": "configs/policy/bypass.schema.json", "configs/policy/slow-checks-ratchet.json": "configs/policy/bypass.schema.json"}
_BYPASS_SCOPES = {"repo", "crate", "module", "file", "path", "docs", "ops", "make", "workflow", "policy"}
_BYPASS_FILES_REGISTRY = Path("configs/policy/bypass-files-registry.json")
_BYPASS_TYPES_REGISTRY = Path("configs/policy/bypass-types.json")
_OPS_BYPASS_LEDGER = Path("ops/_meta/bypass-ledger.json")
_BYPASS_COUNT_BASELINE = Path("configs/policy/bypass-count-baseline.json")
_OPS_META_STRUCTURED_ALLOWLISTS = (
    Path("ops/_meta/cross-area-script-refs-allowlist.json"),
    Path("ops/_meta/layer-contract-literal-allowlist.json"),
    Path("ops/_meta/stack-layer-literal-allowlist.json"),
)


def _bypass_sources_registry(repo_root: Path) -> list[tuple[str, str, bool]]:
    path = repo_root / _BYPASS_FILES_REGISTRY
    if not path.exists():
        return list(_BYPASS_SOURCES)
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return list(_BYPASS_SOURCES)
    rows = payload.get("files", []) if isinstance(payload, dict) else []
    out: list[tuple[str, str, bool]] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        rel = str(row.get("path", "")).strip()
        kind = str(row.get("kind", "")).strip()
        if not rel or not kind:
            continue
        out.append((rel, kind, bool(row.get("requires_metadata", False))))
    return out or list(_BYPASS_SOURCES)


def _bypass_schema_map(repo_root: Path) -> dict[str, str]:
    schema_map = dict(_BYPASS_SCHEMAS)
    path = repo_root / _BYPASS_FILES_REGISTRY
    if not path.exists():
        return schema_map
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return schema_map
    for row in payload.get("files", []) if isinstance(payload, dict) else []:
        if not isinstance(row, dict):
            continue
        rel = str(row.get("path", "")).strip()
        schema = str(row.get("schema", "")).strip()
        if rel and schema:
            schema_map[rel] = schema
    return schema_map
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
    from .public_make_targets import ALLOWED_AREAS, entry_map, load_ownership

    entries = entry_map()
    ownership = load_ownership()
    errors: list[str] = []
    for target, entry in entries.items():
        meta = ownership.get(target)
        if not isinstance(meta, dict):
            errors.append(f"ownership missing for public target: {target}")
            continue
        owner = meta.get("owner")
        area = meta.get("area")
        if not owner:
            errors.append(f"owner missing for public target: {target}")
        if area not in ALLOWED_AREAS:
            errors.append(f"invalid area for {target}: {area} (allowed: {', '.join(sorted(ALLOWED_AREAS))})")
        if area != entry.get("area"):
            errors.append(f"area mismatch for {target}: ssot={entry.get('area')} ownership={area}")
    covered = sum(1 for t in entries if isinstance(ownership.get(t), dict) and ownership[t].get("owner") and ownership[t].get("area"))
    total = len(entries)
    if covered != total:
        errors.append(f"ownership coverage must be 100%: {covered}/{total} ({(covered / total * 100.0) if total else 100.0:.1f}%)")
    return (0 if not errors else 1), errors


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


def check_make_product_mk_wrapper_contract(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "makefiles/product.mk"
    if not path.exists():
        return 1, ["makefiles/product.mk: missing"]
    errors: list[str] = []
    target_count = 0
    target_re = re.compile(r"^(?P<target>[A-Za-z0-9_./-]+):(?:\s|$)")
    for lineno, raw in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
        line = raw.strip()
        m = target_re.match(raw)
        if m and not m.group("target").startswith(".") and not m.group("target").startswith("internal/"):
            target_count += 1
        if re.match(r"^internal/[A-Za-z0-9_./-]+:", line):
            errors.append(f"makefiles/product.mk:{lineno}: internal/* targets are forbidden in product.mk")
        if raw.startswith("\t") and "python3 -c" in raw:
            errors.append(f"makefiles/product.mk:{lineno}: python3 -c is forbidden in wrapper-only product.mk")
        if raw.startswith("\t") and "packages/atlasctl/src/atlasctl/" in raw:
            errors.append(f"makefiles/product.mk:{lineno}: direct python module paths are forbidden in wrapper-only product.mk")
        if raw.startswith("\t") and "atlasctl run ./ops/run/" in raw:
            errors.append(f"makefiles/product.mk:{lineno}: use atlasctl product/ops commands instead of `atlasctl run ./ops/run/...`")
        if raw.startswith("\t") and "./bin/atlasctl" in raw and "./bin/atlasctl product " not in raw:
            errors.append(f"makefiles/product.mk:{lineno}: product.mk must delegate via `./bin/atlasctl product ...` only")
    if target_count > 10:
        errors.append(f"makefiles/product.mk: target budget exceeded ({target_count} > 10)")
    return (0 if not errors else 1), errors


def check_make_no_atlasctl_run_ops_run(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if "atlasctl run ./ops/run/" in body:
            errors.append(f"{rel}:{lineno}: `atlasctl run ./ops/run/...` is forbidden in make recipes")
    return (0 if not errors else 1), sorted(errors)


def check_make_product_migration_complete_no_ops_run(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "makefiles/product.mk"
    if not path.exists():
        return 1, ["makefiles/product.mk: missing"]
    errors: list[str] = []
    for lineno, raw in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
        if raw.startswith("\t") and "ops/run/" in raw:
            errors.append(f"makefiles/product.mk:{lineno}: product migration incomplete; remove ops/run reference")
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
            if (
                target.startswith("internal/")
                or target.startswith("atlasctl/internal")
                or target in {"verification", "_verification-run"}
            ):
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


def check_makefiles_wrappers_only_all(repo_root: Path) -> tuple[int, list[str]]:
    # Explicit invariant alias requested by CI/governance program.
    return check_make_wrapper_purity(repo_root)


def check_make_no_python_module_invocation(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if _ATLASCTL_MODULE_RE.search(body):
            errors.append(f"{rel}:{lineno}: forbidden python module invocation in make recipe (`python -m atlasctl.cli`)")
    return (0 if not errors else 1), sorted(errors)




def _iter_make_targets_in_file(repo_root: Path, rel: str) -> list[str]:
    path = repo_root / rel
    if not path.exists():
        return []
    out: list[str] = []
    target_re = re.compile(r"^(?P<target>[A-Za-z0-9_./-]+):(?:\s|$)")
    for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        m = target_re.match(line)
        if not m:
            continue
        target = m.group("target")
        if target.startswith(".") or target.startswith("internal/"):
            continue
        out.append(target)
    return out


def check_make_ops_product_no_tool_tokens(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    forbidden = (" bash", "python ", "python3", "kubectl", "helm", "docker")
    for rel in ("makefiles/ops.mk", "makefiles/product.mk"):
        for line_rel, lineno, body in _iter_make_recipe_lines(repo_root):
            if line_rel != rel:
                continue
            if "./bin/atlasctl" in body:
                # allow atlasctl subcommands that mention docker/helm/kubectl lexically
                continue
            for token in forbidden:
                if token in f" {body}":
                    errors.append(f"{rel}:{lineno}: forbidden tool token in wrapper recipe (`{token.strip()}`)")
                    break
    return (0 if not errors else 1), sorted(errors)


def check_make_ops_product_atlasctl_only_delegation(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    expected = {"makefiles/ops.mk": "./bin/atlasctl ops ", "makefiles/product.mk": "./bin/atlasctl product "}
    for rel, prefix in expected.items():
        for line_rel, lineno, body in _iter_make_recipe_lines(repo_root):
            if line_rel != rel:
                continue
            if not body.startswith("@./bin/atlasctl ") and not body.startswith("./bin/atlasctl "):
                errors.append(f"{rel}:{lineno}: wrapper recipe must delegate via ./bin/atlasctl")
                continue
            if prefix not in body:
                errors.append(f"{rel}:{lineno}: wrapper recipe must delegate via `{prefix.strip()}...`")
    return (0 if not errors else 1), sorted(errors)


def check_make_ops_product_ownership_complete(repo_root: Path) -> tuple[int, list[str]]:
    ownership_path = repo_root / "configs/make/ownership.json"
    if not ownership_path.exists():
        return 1, ["configs/make/ownership.json: missing"]
    ownership_payload = json.loads(ownership_path.read_text(encoding="utf-8"))
    ownership: dict[str, object] = {}
    if isinstance(ownership_payload, dict):
        for key, value in ownership_payload.items():
            if key == "targets":
                continue
            if isinstance(value, dict):
                ownership[key] = value
        nested = ownership_payload.get("targets")
        if isinstance(nested, dict):
            for key, value in nested.items():
                if isinstance(value, dict):
                    ownership[key] = value
    errors: list[str] = []
    for rel in ("makefiles/ops.mk", "makefiles/product.mk"):
        for target in _iter_make_targets_in_file(repo_root, rel):
            if target not in ownership:
                errors.append(f"{rel}: ownership missing for target `{target}`")
    return (0 if not errors else 1), sorted(errors)


def check_make_removed_makefiles_unreferenced(repo_root: Path) -> tuple[int, list[str]]:
    cfg = repo_root / "configs/make/retired-makefiles.json"
    if not cfg.exists():
        return 1, ["configs/make/retired-makefiles.json: missing"]
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    retired = [str(x).strip() for x in payload.get("retired_makefiles", []) if str(x).strip()] if isinstance(payload, dict) else []
    errors: list[str] = []
    scan_roots = [repo_root / ".github", repo_root / "docs", repo_root / "makefiles", repo_root / "packages/atlasctl/src", repo_root / "packages/atlasctl/tests", repo_root / "configs"]
    for relpath in retired:
        for base in scan_roots:
            if not base.exists():
                continue
            for path in base.rglob("*"):
                if not path.is_file():
                    continue
                if path == cfg:
                    continue
                text = path.read_text(encoding="utf-8", errors="ignore")
                if relpath in text:
                    errors.append(f"{path.relative_to(repo_root).as_posix()}: references retired makefile `{relpath}`")
    return (0 if not errors else 1), sorted(errors)


def check_ci_workflows_call_only_delegating_make_targets(repo_root: Path) -> tuple[int, list[str]]:
    return check_ci_workflows_call_make_and_make_calls_atlasctl(repo_root)

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
    ops_suites_path = repo_root / "configs" / "ops" / "suites.json"
    if ops_suites_path.exists():
        try:
            payload = json.loads(ops_suites_path.read_text(encoding="utf-8"))
            for row in payload.get("suites", []) if isinstance(payload, dict) else []:
                if isinstance(row, dict):
                    name = str(row.get("name", "")).strip()
                    if name:
                        known.add(name)
        except Exception:
            pass
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
    for rel_path, kind, requires_metadata in _bypass_sources_registry(repo_root):
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
        if kind == "toml":
            # Inventory-only support for TOML allowlists (entry metadata often not structured like policy bypass JSON).
            text = path.read_text(encoding="utf-8", errors="ignore")
            line_no = 0
            for line in text.splitlines():
                line_no += 1
                stripped = line.strip()
                if not stripped or stripped.startswith("#") or stripped.startswith("["):
                    continue
                if "=" not in stripped:
                    continue
                key = stripped.split("=", 1)[0].strip()
                entries.append({"source": rel_path, "key": key or f"{rel_path}#{line_no}", "requires_metadata": False})
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        if rel_path.endswith("temporary-shims.json") and isinstance(payload, dict):
            for item in payload.get("shims", []):
                if not isinstance(item, dict):
                    continue
                item_key = str(item.get("id") or item.get("command") or f"shim#{len(entries)+1}").strip()
                entries.append(
                    {
                        "source": rel_path,
                        "key": item_key,
                        "policy_name": "temporary-shim",
                        "scope": "ops",
                        "owner": str(item.get("owner", "")).strip(),
                        "issue_id": str(item.get("approval_id", "")).strip(),
                        "expiry": str(item.get("expires_on", "")).strip(),
                        "justification": str(item.get("reason", "")).strip(),
                        "removal_plan": str(item.get("removal_plan", "")).strip() or "Remove temporary shim after migration parity is proven.",
                        "replacement_mechanism": str(item.get("replacement_mechanism", "")).strip(),
                        "severity": str(item.get("severity", "")).strip(),
                        "requires_metadata": True,
                    }
                )
            continue
        if rel_path.endswith("budget-relaxations-audit.json") and isinstance(payload, dict):
            for list_key in ("active_relaxations", "expired_relaxations"):
                for item in payload.get(list_key, []) if isinstance(payload.get(list_key), list) else []:
                    item_key = str(item.get("id") if isinstance(item, dict) else item).strip() or f"{list_key}#{len(entries)+1}"
                    entries.append(
                        {
                            "source": rel_path,
                            "key": item_key,
                            "policy_name": "budget-audit",
                            "scope": "ops",
                            "owner": "ops",
                            "issue_id": "",
                            "expiry": "",
                            "justification": list_key,
                            "removal_plan": "",
                            "requires_metadata": False,
                        }
                    )
            continue
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
        for list_key in ("entries", "exceptions", "relaxations", "allow", "allowlist", "undeclared_import_allowlist", "optional_dependency_usage_allowlist", "internal_third_party_allowlist"):
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
                    "expiry": str(item.get("expiry") or item.get("expires_on") or item.get("expires_at") or "").strip(),
                    "justification": str(item.get("justification") or item.get("reason") or "").strip(),
                    "removal_plan": str(item.get("removal_plan", "")).strip(),
                    "bypass_id": str(item.get("bypass_id", "")).strip(),
                    "task_id": str(item.get("task_id", "")).strip(),
                    "created_at": str(item.get("created_at", "")).strip(),
                    "contract_test": str(item.get("contract_test", "")).strip(),
                    "necessity_test": str(item.get("necessity_test", "")).strip(),
                    "severity": str(item.get("severity", "")).strip(),
                    "replacement_mechanism": str(item.get("replacement_mechanism", "")).strip(),
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
    registry_types = {}
    types_path = repo_root / _BYPASS_TYPES_REGISTRY
    if types_path.exists():
        try:
            types_payload = json.loads(types_path.read_text(encoding="utf-8"))
            registry_types = {str(item) for item in types_payload.get("types", [])} if isinstance(types_payload, dict) else set()
        except Exception:
            registry_types = {}
    registry_payload = json.loads((repo_root / _BYPASS_FILES_REGISTRY).read_text(encoding="utf-8")) if (repo_root / _BYPASS_FILES_REGISTRY).exists() else {}
    type_by_path = {
        str(row.get("path", "")).strip(): str(row.get("type", "")).strip()
        for row in (registry_payload.get("files", []) if isinstance(registry_payload, dict) else [])
        if isinstance(row, dict)
    }
    for rel_path, kind, _ in _bypass_sources_registry(repo_root):
        files.append(
            {
                "path": rel_path,
                "exists": (repo_root / rel_path).exists(),
                "kind": kind,
                "type": type_by_path.get(rel_path, ""),
                "entry_count": counts.get(rel_path, 0),
            }
        )
    return {
        "schema_version": 1,
        "files": files,
        "entry_count": len(entries),
        "entries": entries,
        "errors": [],
        "registry": {
            "files_registry": _BYPASS_FILES_REGISTRY.as_posix(),
            "types_registry": _BYPASS_TYPES_REGISTRY.as_posix(),
            "types": sorted(registry_types) if isinstance(registry_types, set) else [],
            "ops_bypass_ledger": _OPS_BYPASS_LEDGER.as_posix(),
        },
    }


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
    by_check.update({k: [] for k in ("severity", "replacement", "severity_expiry", "forbidden_policy")})
    known_files = {row[0] for row in _bypass_sources_registry(repo_root)} | {"configs/policy/migration_exceptions.json", "configs/policy/checks-registry-transition.json", "configs/policy/check-id-migration.json", "configs/policy/forbidden-adjectives-approvals.json", "configs/policy/check_speed_approvals.json", "configs/policy/checks-shell-direct-allowlist.json", _BYPASS_FILES_REGISTRY.as_posix(), _BYPASS_TYPES_REGISTRY.as_posix(), "configs/policy/bypass-new-entry-approvals.json"}
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
    known_policy.update({"allow", "allowlist", "exceptions", "relaxations", "undeclared_import_allowlist", "optional_dependency_usage_allowlist", "internal_third_party_allowlist", "temporary-shim"})
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
        if str(row.get("replacement_mechanism", "")).strip() == "":
            by_check["replacement"].append(f"{source}:{key}: replacement_mechanism is required")
        if scope not in _BYPASS_SCOPES:
            by_check["scope"].append(f"{source}:{key}: invalid scope `{scope}`")
        if not policy_name or policy_name not in known_policy:
            by_check["policy"].append(f"{source}:{key}: unknown policy/rule `{policy_name}`")
        severity = str(row.get("severity", "")).strip()
        if severity not in {"P0", "P1", "P2", "P3"}:
            by_check["severity"].append(f"{source}:{key}: severity must be one of P0/P1/P2/P3")
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
        if severity in {"P0", "P1"} and delta > 30:
            by_check["severity_expiry"].append(f"{source}:{key}: {severity} expiry horizon {delta}d exceeds 30d")
        lname = policy_name.lower()
        if any(token in lname for token in ("image", "pinning", "unpinned", "network", "script")):
            by_check["forbidden_policy"].append(f"{source}:{key}: bypass forbidden for policy/rule `{policy_name}`")
    try:
        import jsonschema
        schema_map = _bypass_schema_map(repo_root)
        for rel_path, _, _ in _bypass_sources_registry(repo_root):
            schema_rel = schema_map.get(rel_path)
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


def _ops_bypass_ledger(repo_root: Path) -> tuple[dict[str, object], dict[str, dict[str, object]]]:
    path = repo_root / _OPS_BYPASS_LEDGER
    if not path.exists():
        return {}, {}
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = payload.get("entries", []) if isinstance(payload, dict) else []
    by_id: dict[str, dict[str, object]] = {}
    for row in rows:
        if isinstance(row, dict):
            rid = str(row.get("id", "")).strip()
            if rid:
                by_id[rid] = row
    return payload if isinstance(payload, dict) else {}, by_id


def _iter_ops_meta_bypass_entries(repo_root: Path) -> list[tuple[str, dict[str, object]]]:
    rows: list[tuple[str, dict[str, object]]] = []
    for rel in _OPS_META_STRUCTURED_ALLOWLISTS:
        path = repo_root / rel
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        for row in payload.get("entries", []) if isinstance(payload, dict) else []:
            if isinstance(row, dict):
                rows.append((rel.as_posix(), row))
    return rows
def _bypass_readme_errors(repo_root: Path) -> dict[str, list[str]]:
    path = repo_root / "configs/policy/README.md"
    if not path.exists():
        return {"complete": ["configs/policy/README.md missing"], "sorted": ["configs/policy/README.md missing"]}
    text = path.read_text(encoding="utf-8")
    refs = sorted(set(re.findall(r"`((?:configs/(?:policy|layout|ops|security)|ops/(?:_artifacts|_meta))/[^`]+)`", text)))
    expected = sorted(row[0] for row in _bypass_sources_registry(repo_root))
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


def check_policies_bypass_inventory_present(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    errors: list[str] = []
    if not (repo_root / _BYPASS_FILES_REGISTRY).exists():
        errors.append(f"missing bypass files registry: {_BYPASS_FILES_REGISTRY.as_posix()}")
    if not (repo_root / _BYPASS_TYPES_REGISTRY).exists():
        errors.append(f"missing bypass types registry: {_BYPASS_TYPES_REGISTRY.as_posix()}")
    if not payload.get("files"):
        errors.append("bypass inventory files list must be non-empty")
    return _res(errors)


def check_policies_bypass_inventory_deterministic(repo_root: Path) -> tuple[int, list[str]]:
    p1 = collect_bypass_inventory(repo_root)
    p2 = collect_bypass_inventory(repo_root)
    if json.dumps(p1, sort_keys=True) != json.dumps(p2, sort_keys=True):
        return 1, ["bypass inventory payload is nondeterministic across repeated runs"]
    file_rows = p1.get("files", [])
    if isinstance(file_rows, list):
        paths = [str(row.get("path", "")) for row in file_rows if isinstance(row, dict)]
        if paths != sorted(paths):
            return 1, ["bypass inventory files must be sorted by path"]
    return 0, []


def check_policies_bypass_has_owner(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["owner"] + _bypass_errors(repo_root)["meta"])
def check_policies_bypass_has_expiry(repo_root: Path) -> tuple[int, list[str]]: return _res(_bypass_errors(repo_root)["expiry"] + _bypass_errors(repo_root)["meta"])


def check_policies_bypass_has_reason(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    errors = list(_bypass_errors(repo_root)["justification"])
    for row in payload.get("entries", []):
        if not isinstance(row, dict) or not bool(row.get("requires_metadata", False)):
            continue
        just = str(row.get("justification", "")).strip().lower()
        src = str(row.get("source", ""))
        key = str(row.get("key", ""))
        if just in {"because", "because needed", "temporary"} or just.startswith("because "):
            errors.append(f"{src}:{key}: non-specific justification (`because`) is forbidden")
    return _res(errors)


def check_policies_bypass_has_ticket_or_doc_ref(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    errors: list[str] = []
    for row in payload.get("entries", []):
        if not isinstance(row, dict) or not bool(row.get("requires_metadata", False)):
            continue
        issue = str(row.get("issue_id", "")).strip()
        just = str(row.get("justification", "")).strip()
        src = str(row.get("source", ""))
        key = str(row.get("key", ""))
        has_doc_ref = ("docs/" in just) or ("ADR-" in just) or ("policy:" in just.lower())
        if not issue and not has_doc_ref:
            errors.append(f"{src}:{key}: require issue_id or local doc reference in justification")
    return _res(errors)


def check_policies_bypass_budget_trend(repo_root: Path) -> tuple[int, list[str]]: return check_policies_bypass_budget(repo_root)
def check_policies_bypass_inventory_schema_valid(repo_root: Path) -> tuple[int, list[str]]: return check_policies_bypass_schema_valid(repo_root)


def check_policies_bypass_new_entries_forbidden(repo_root: Path) -> tuple[int, list[str]]:
    baseline_path = repo_root / _BYPASS_COUNT_BASELINE
    approvals_path = repo_root / "configs/policy/bypass-new-entry-approvals.json"
    if not baseline_path.exists():
        return 1, [f"missing baseline file: {_BYPASS_COUNT_BASELINE.as_posix()}"]
    baseline = int(json.loads(baseline_path.read_text(encoding="utf-8")).get("max_entries", 0))
    current = int(collect_bypass_inventory(repo_root).get("entry_count", 0))
    approvals_payload = json.loads(approvals_path.read_text(encoding="utf-8")) if approvals_path.exists() else {"approvals": []}
    approvals = approvals_payload.get("approvals", []) if isinstance(approvals_payload, dict) else []
    if current <= baseline:
        return 0, []
    if not approvals:
        return 1, [f"new bypass entries forbidden: current={current} baseline={baseline}; add explicit approvals in configs/policy/bypass-new-entry-approvals.json"]
    return 0, []


def check_policies_bypass_entry_paths_exist(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    errors: list[str] = []
    for row in payload.get("entries", []):
        if not isinstance(row, dict):
            continue
        key = str(row.get("key", "")).strip()
        src = str(row.get("source", "")).strip()
        if "/" not in key or any(ch in key for ch in "*?[]"):
            continue
        candidate = repo_root / key
        if not candidate.exists():
            errors.append(f"{src}:{key}: referenced path does not exist")
    return _res(errors)


def check_policies_bypass_entry_matches_nothing(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    files = [p.relative_to(repo_root).as_posix() for p in repo_root.rglob("*") if p.is_file()]
    errors: list[str] = []
    for row in payload.get("entries", []):
        if not isinstance(row, dict):
            continue
        key = str(row.get("key", "")).strip()
        src = str(row.get("source", "")).strip()
        if not any(ch in key for ch in "*?[]"):
            continue
        if not any(fnmatch.fnmatch(path, key) for path in files):
            errors.append(f"{src}:{key}: wildcard bypass matches no files")
    return _res(errors)


def check_policies_bypass_entry_matches_too_broad(repo_root: Path) -> tuple[int, list[str]]:
    payload = collect_bypass_inventory(repo_root)
    files = [p.relative_to(repo_root).as_posix() for p in repo_root.rglob("*") if p.is_file()]
    errors: list[str] = []
    for row in payload.get("entries", []):
        if not isinstance(row, dict):
            continue
        key = str(row.get("key", "")).strip()
        src = str(row.get("source", "")).strip()
        if not any(ch in key for ch in "*?[]"):
            continue
        matches = [path for path in files if fnmatch.fnmatch(path, key)]
        if len(matches) > 50:
            errors.append(f"{src}:{key}: wildcard bypass matches too broadly ({len(matches)} files)")
    return _res(errors)


def check_policies_bypass_severity_levels(repo_root: Path) -> tuple[int, list[str]]:
    return _res(_bypass_errors(repo_root)["severity"])


def check_policies_bypass_replacement_mechanism(repo_root: Path) -> tuple[int, list[str]]:
    return _res(_bypass_errors(repo_root)["replacement"])


def check_policies_bypass_severity_expiry_windows(repo_root: Path) -> tuple[int, list[str]]:
    return _res(_bypass_errors(repo_root)["severity_expiry"])


def check_policies_bypass_no_security_or_pinning(repo_root: Path) -> tuple[int, list[str]]:
    return _res([e for e in _bypass_errors(repo_root)["forbidden_policy"] if any(t in e.lower() for t in ("security", "image", "pin"))])


def check_policies_bypass_no_unpinned_images(repo_root: Path) -> tuple[int, list[str]]:
    return _res([e for e in _bypass_errors(repo_root)["forbidden_policy"] if "unpinned" in e.lower() or "image" in e.lower()])


def check_policies_bypass_no_network_guard(repo_root: Path) -> tuple[int, list[str]]:
    return _res([e for e in _bypass_errors(repo_root)["forbidden_policy"] if "network" in e.lower()])


def check_policies_bypass_no_direct_script_runs(repo_root: Path) -> tuple[int, list[str]]:
    return _res([e for e in _bypass_errors(repo_root)["forbidden_policy"] if "script" in e.lower()])


def check_policies_bypass_inventory_growth_requires_ticket(repo_root: Path) -> tuple[int, list[str]]:
    return check_policies_bypass_new_entries_forbidden(repo_root)


def check_ops_bypass_ledger_refs_present(repo_root: Path) -> tuple[int, list[str]]:
    _, by_id = _ops_bypass_ledger(repo_root)
    errors: list[str] = []
    for rel, row in _iter_ops_meta_bypass_entries(repo_root):
        bid = str(row.get("bypass_id", "")).strip()
        key = str(row.get("id") or row.get("path") or "").strip()
        if not bid:
            errors.append(f"{rel}:{key}: missing bypass_id")
            continue
        if bid not in by_id:
            errors.append(f"{rel}:{key}: bypass_id `{bid}` missing from {_OPS_BYPASS_LEDGER.as_posix()}")
    return _res(errors)


def check_ops_bypass_allowlist_files_have_owner(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel in _OPS_META_STRUCTURED_ALLOWLISTS:
        p = repo_root / rel
        if not p.exists():
            errors.append(f"missing allowlist file: {rel.as_posix()}")
            continue
        payload = json.loads(p.read_text(encoding="utf-8"))
        owner = str(payload.get("owner", "")).strip() if isinstance(payload, dict) else ""
        if not owner:
            errors.append(f"{rel.as_posix()}: top-level owner is required")
    return _res(errors)


def check_ops_bypass_ledger_has_expiry(repo_root: Path) -> tuple[int, list[str]]:
    payload, _ = _ops_bypass_ledger(repo_root)
    rows = payload.get("entries", []) if isinstance(payload, dict) else []
    errors = [f"{_OPS_BYPASS_LEDGER.as_posix()}:{i+1}: missing expires_at" for i, row in enumerate(rows) if isinstance(row, dict) and not str(row.get("expires_at", "")).strip()]
    return _res(errors)


def check_ops_bypass_ledger_expiry_not_past(repo_root: Path) -> tuple[int, list[str]]:
    payload, _ = _ops_bypass_ledger(repo_root)
    rows = payload.get("entries", []) if isinstance(payload, dict) else []
    today = dt.date.today()
    errors: list[str] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        rid = str(row.get("id", "")).strip()
        raw = str(row.get("expires_at", "")).strip()
        if not raw:
            continue
        try:
            d = dt.date.fromisoformat(raw)
        except ValueError:
            errors.append(f"{_OPS_BYPASS_LEDGER.as_posix()}:{rid}: invalid expires_at `{raw}`")
            continue
        if d < today:
            errors.append(f"{_OPS_BYPASS_LEDGER.as_posix()}:{rid}: expired on {raw}")
    return _res(errors)


def check_ops_bypass_ledger_task_ids(repo_root: Path) -> tuple[int, list[str]]:
    payload, _ = _ops_bypass_ledger(repo_root)
    rows = payload.get("entries", []) if isinstance(payload, dict) else []
    errors: list[str] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        rid = str(row.get("id", "")).strip()
        if not str(row.get("task_id", "")).strip():
            errors.append(f"{_OPS_BYPASS_LEDGER.as_posix()}:{rid}: missing task_id")
    return _res(errors)


def check_ops_bypass_ledger_sorted(repo_root: Path) -> tuple[int, list[str]]:
    payload, _ = _ops_bypass_ledger(repo_root)
    rows = payload.get("entries", []) if isinstance(payload, dict) else []
    ids = [str(r.get("id", "")) for r in rows if isinstance(r, dict)]
    if ids != sorted(ids):
        return 1, [f"{_OPS_BYPASS_LEDGER.as_posix()}: entries must be sorted by id"]
    return 0, []


def check_ops_bypass_ledger_domains(repo_root: Path) -> tuple[int, list[str]]:
    allowed = {"stack", "k8s", "obs", "load", "e2e", "datasets", "ops", "shared"}
    payload, ledger = _ops_bypass_ledger(repo_root)
    if payload.get("errors"):
        return 1, [str(x) for x in payload.get("errors", [])]
    out: list[str] = []
    for entry in ledger.values():
        domain = str(entry.get("domain", "")).strip()
        if not domain:
            out.append(f"{_OPS_BYPASS_LEDGER.as_posix()}:{entry.get('id')}: domain is required")
        elif domain not in allowed:
            out.append(f"{_OPS_BYPASS_LEDGER.as_posix()}:{entry.get('id')}: invalid domain `{domain}`")
    return (0 if not out else 1), out


def check_policies_culprits_empty_on_release_tags(repo_root: Path) -> tuple[int, list[str]]:
    ref_type = os.environ.get("GITHUB_REF_TYPE", "").strip().lower()
    ref_name = os.environ.get("GITHUB_REF_NAME", "").strip()
    ref = os.environ.get("GITHUB_REF", "").strip()
    is_tag = ref_type == "tag" or ref.startswith("refs/tags/")
    if not is_tag:
        return 0, []
    payload = collect_bypass_inventory(repo_root)
    count = int(payload.get("entry_count", 0))
    if count == 0:
        return 0, []
    label = ref_name or ref or "<tag>"
    return 1, [f"release tag `{label}` requires zero policy culprits; found {count} bypass entries"]


def check_policies_culprits_nonincrease_on_main(repo_root: Path) -> tuple[int, list[str]]:
    ref_name = os.environ.get("GITHUB_REF_NAME", "").strip()
    ref = os.environ.get("GITHUB_REF", "").strip()
    event = os.environ.get("GITHUB_EVENT_NAME", "").strip()
    is_main = ref_name == "main" or ref.endswith("/main")
    if not is_main and event not in {"schedule", "workflow_dispatch"}:
        return 0, []
    return check_policies_bypass_count_trend(repo_root)


def check_policies_bypass_tokens_linked(repo_root: Path) -> tuple[int, list[str]]:
    token_re = re.compile(r"\b(?:ALLOWLIST|TEMP|HACK)\b")
    id_re = re.compile(r"\b(?:OPS-BYPASS-\d{4}|ATLAS-(?:EXC|LAYER-EXC)-\d{4})\b")
    errors: list[str] = []
    roots = (repo_root / "ops/_meta", repo_root / "configs/policy")
    for root in roots:
        if not root.exists():
            continue
        for path in sorted(root.rglob("*")):
            if not path.is_file() or path.suffix not in {".json", ".txt", ".md"}:
                continue
            rel = path.relative_to(repo_root).as_posix()
            for i, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
                if not token_re.search(line):
                    continue
                if id_re.search(line):
                    continue
                # Allow schema/property names and generated markdown headings.
                if '"ALLOWLIST"' in line or "'ALLOWLIST'" in line:
                    continue
                errors.append(f"{rel}:{i}: token ALLOWLIST/TEMP/HACK must link to bypass ledger id")
    return (0 if not errors else 1), errors


def check_ops_allowlisted_literals_have_contract_tests(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel in (_OPS_META_STRUCTURED_ALLOWLISTS[1], _OPS_META_STRUCTURED_ALLOWLISTS[2]):
        path = repo_root / rel
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        for row in payload.get("entries", []) if isinstance(payload, dict) else []:
            if not isinstance(row, dict):
                continue
            key = str(row.get("id") or row.get("path") or "").strip()
            for field in ("contract_test", "necessity_test"):
                val = str(row.get(field, "")).strip()
                if not val:
                    errors.append(f"{rel.as_posix()}:{key}: missing {field}")
                    continue
                if not (repo_root / val).exists():
                    errors.append(f"{rel.as_posix()}:{key}: {field} path does not exist ({val})")
    return _res(errors)


def check_ops_bypass_entries_have_necessity_tests(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel, row in _iter_ops_meta_bypass_entries(repo_root):
        key = str(row.get("id") or row.get("path") or "").strip()
        val = str(row.get("necessity_test", "")).strip()
        if not val:
            errors.append(f"{rel}:{key}: missing necessity_test")
            continue
        if not (repo_root / val).exists():
            errors.append(f"{rel}:{key}: necessity_test path does not exist ({val})")
    return _res(errors)
