from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

from ...commands.dev.make.public_targets import public_names
from ..repo.native import (
    check_make_no_direct_python_script_invocations,
    check_make_scripts_references,
)

_MAKE_RECIPE_RE = re.compile(r"^\t(?P<body>.*)$")
_SCRIPT_PATH_RE = re.compile(r"(^|\s)(?:\./)?(?:ops|scripts|packages/atlasctl/src/atlasctl)/[^\s]+")
_BASH_OPS_RE = re.compile(r"(?:^|\s)(?:bash|sh)\s+(?:\./)?ops/[^\s]+")
_WRITE_RE = re.compile(r"(?:^|\s)(?:cp\s+[^\n]*\s+|mv\s+[^\n]*\s+|cat\s+>\s*|tee\s+|mkdir\s+-p\s+|touch\s+|>\s*|>>\s*)([^\s\"';]+)")


def _iter_make_recipe_lines(repo_root: Path) -> list[tuple[str, int, str]]:
    rows: list[tuple[str, int, str]] = []
    files = [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]
    for path in files:
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            match = _MAKE_RECIPE_RE.match(line)
            if not match:
                continue
            body = match.group("body").strip()
            if not body or body.startswith("#"):
                continue
            rows.append((rel, lineno, body))
    return rows


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
    tool_path = repo_root / "packages/atlasctl/src/atlasctl/checks/layout/public_surface/tools"
    env = dict(**__import__("os").environ)
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{tool_path}:{existing}" if existing else str(tool_path)
    proc = subprocess.run(
        ["python3", script],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )
    if proc.returncode == 0:
        return 0, []
    output = (proc.stderr or proc.stdout or "check failed").strip()
    return 1, [output]


def check_make_ci_entrypoints_contract(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/layout/workflows/check_ci_entrypoints.py")


def check_make_public_targets_documented(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(
        repo_root,
        "packages/atlasctl/src/atlasctl/checks/layout/public_surface/checks/check_public_targets_documented.py",
    )


def check_make_target_ownership_complete(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/make/impl/check_make_target_ownership.py")


def check_make_target_boundaries_enforced(repo_root: Path) -> tuple[int, list[str]]:
    return _run_script(repo_root, "packages/atlasctl/src/atlasctl/checks/make/impl/check_makefile_target_boundaries.py")


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
    for rel, lineno, body in _iter_make_recipe_lines(repo_root):
        if "atlasctl" in body or "$(ATLAS_SCRIPTS)" in body or "$(MAKE)" in body:
            continue
        if body.startswith("@echo ") or body.startswith("echo "):
            continue
        msg = f"{rel}:{lineno}: make recipe bypasses atlasctl wrapper"
        if msg in exceptions:
            continue
        errors.append(msg)
    return (0 if not errors else 1), sorted(errors)


def check_ci_workflows_call_make_and_make_calls_atlasctl(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    workflows = sorted((repo_root / ".github" / "workflows").glob("*.yml"))
    for wf in workflows:
        text = wf.read_text(encoding="utf-8", errors="ignore")
        run_lines = [line.strip() for line in text.splitlines() if line.strip().startswith("run:")]
        if not run_lines:
            continue
        make_lines = [line for line in run_lines if re.search(r"\brun:\s*make\b", line)]
        if not make_lines:
            errors.append(f"{wf.relative_to(repo_root).as_posix()}: workflow must call make entrypoints")
        direct_atlasctl = [line for line in run_lines if "atlasctl " in line and "make " not in line]
        if direct_atlasctl:
            errors.append(f"{wf.relative_to(repo_root).as_posix()}: workflow run lines must call make, not atlasctl directly")
    return (0 if not errors else 1), sorted(errors)


def check_public_make_targets_map_to_atlasctl(repo_root: Path) -> tuple[int, list[str]]:
    root_mk = repo_root / "makefiles" / "root.mk"
    if not root_mk.exists():
        return 1, ["makefiles/root.mk missing"]
    text = root_mk.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    for target in public_names():
        pattern = re.compile(rf"(?m)^{re.escape(target)}:\s.*?(?:\n(?:\t.*\n?)*)")
        match = pattern.search(text)
        if not match:
            errors.append(f"public target missing from makefiles/root.mk: {target}")
            continue
        block = match.group(0)
        if "atlasctl" not in block and "$(ATLAS_SCRIPTS)" not in block:
            errors.append(f"public target must delegate to atlasctl: {target}")
    return (0 if not errors else 1), sorted(errors)
