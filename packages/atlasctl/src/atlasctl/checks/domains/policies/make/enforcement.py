from __future__ import annotations

import json
import re
import runpy
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
    "makefiles/registry.mk",
    "makefiles/env.mk",
    "makefiles/root.mk",
)
_ROOT_MK_MAX_LOC = 900
_ROOT_MK_MAX_TARGETS = 220


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
