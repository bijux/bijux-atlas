from __future__ import annotations

import importlib
import re
import subprocess
import sys
from pathlib import Path


_REQUIRED_TOML_BLOCKS = (
    "[project]",
    "[project.scripts]",
    "[project.optional-dependencies]",
    "[tool.ruff]",
    "[tool.pytest.ini_options]",
    "[tool.mypy]",
    "[tool.coverage.run]",
)

_FORBIDDEN_TOOL_CONFIGS = (
    "ruff.toml",
    ".ruff.toml",
    "pytest.ini",
    "mypy.ini",
    ".flake8",
    "tox.ini",
)

_REQUIRED_OPTIONAL_DEP_GROUPS = ("dev", "test", "ops", "docs")
_ALLOWED_TOOL_PREFIXES = {"setuptools", "pytest", "ruff", "mypy", "coverage"}
_DEPS_WORKFLOW_MARKER = "Workflow: pip-tools (requirements.in + requirements.lock.txt)"
_REQ_ALLOWED = {"requirements.in", "requirements.lock.txt"}


def _read_pyproject(repo_root: Path) -> str:
    return (repo_root / "packages/atlasctl/pyproject.toml").read_text(encoding="utf-8")


def check_pyproject_required_blocks(repo_root: Path) -> tuple[int, list[str]]:
    text = _read_pyproject(repo_root)
    errors = [f"missing required pyproject block: {name}" for name in _REQUIRED_TOML_BLOCKS if name not in text]
    return (0 if not errors else 1), errors


def check_pyproject_no_duplicate_tool_config(repo_root: Path) -> tuple[int, list[str]]:
    package_root = repo_root / "packages/atlasctl"
    errors = [
        f"forbidden tool config beside pyproject: packages/atlasctl/{name}"
        for name in _FORBIDDEN_TOOL_CONFIGS
        if (package_root / name).exists()
    ]
    return (0 if not errors else 1), errors


def check_console_script_entry(repo_root: Path) -> tuple[int, list[str]]:
    text = _read_pyproject(repo_root)
    m = re.search(r"(?m)^atlasctl\s*=\s*\"([A-Za-z0-9_\\.]+):([A-Za-z0-9_]+)\"\s*$", text)
    if not m:
        return 1, ["missing [project.scripts] atlasctl entry in pyproject.toml"]
    module_name, attr_name = m.group(1), m.group(2)
    errors: list[str] = []
    try:
        module = importlib.import_module(module_name)
        target = getattr(module, attr_name, None)
        if not callable(target):
            errors.append(f"console script target is not callable: {module_name}:{attr_name}")
    except Exception as exc:  # pragma: no cover
        errors.append(f"console script target import failed: {module_name}:{attr_name} ({exc})")
    return (0 if not errors else 1), errors


def check_python_module_help(repo_root: Path) -> tuple[int, list[str]]:
    env = {"PYTHONPATH": str(repo_root / "packages/atlasctl/src")}
    proc = subprocess.run(
        [sys.executable, "-m", "atlasctl", "--help"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )
    if proc.returncode == 0:
        return 0, []
    msg = proc.stderr.strip() or proc.stdout.strip() or "python -m atlasctl --help failed"
    return 1, [msg]


def check_optional_dependency_groups(repo_root: Path) -> tuple[int, list[str]]:
    text = _read_pyproject(repo_root)
    block = re.search(r"(?ms)^\[project\.optional-dependencies\]\n(.*?)(?:^\[|\Z)", text)
    body = block.group(1) if block else ""
    declared = set(re.findall(r"(?m)^([A-Za-z0-9_-]+)\s*=", body))
    errors = [f"missing required optional dependency group: {name}" for name in _REQUIRED_OPTIONAL_DEP_GROUPS if name not in declared]
    return (0 if not errors else 1), errors


def check_pyproject_minimalism(repo_root: Path) -> tuple[int, list[str]]:
    text = _read_pyproject(repo_root)
    tool = set(re.findall(r"(?m)^\[tool\.([A-Za-z0-9_-]+)", text))
    errors = [
        f"unknown pyproject tool section: [tool.{name}]"
        for name in sorted(tool)
        if name not in _ALLOWED_TOOL_PREFIXES
    ]
    return (0 if not errors else 1), errors


def check_deps_workflow_doc(repo_root: Path) -> tuple[int, list[str]]:
    deps_doc = repo_root / "packages/atlasctl/docs/deps.md"
    req_in = repo_root / "packages/atlasctl/requirements.in"
    req_lock = repo_root / "packages/atlasctl/requirements.lock.txt"
    errors: list[str] = []
    if not deps_doc.exists():
        errors.append("missing docs/deps.md")
        return 1, errors
    text = deps_doc.read_text(encoding="utf-8")
    if _DEPS_WORKFLOW_MARKER not in text:
        errors.append("docs/deps.md missing canonical workflow marker")
    if not req_in.exists():
        errors.append("missing requirements.in for pip-tools workflow")
    if not req_lock.exists():
        errors.append("missing requirements.lock.txt for pip-tools workflow")
    return (0 if not errors else 1), errors


def check_env_docs_present(repo_root: Path) -> tuple[int, list[str]]:
    env_doc = repo_root / "packages/atlasctl/docs/env.md"
    if not env_doc.exists():
        return 1, ["missing docs/env.md"]
    text = env_doc.read_text(encoding="utf-8")
    required = ("BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT", "ATLASCTL_ARTIFACT_ROOT", "XDG_CACHE_HOME")
    errors = [f"docs/env.md missing env var: {name}" for name in required if name not in text]
    return (0 if not errors else 1), errors


def check_requirements_artifact_policy(repo_root: Path) -> tuple[int, list[str]]:
    package_root = repo_root / "packages/atlasctl"
    found = sorted(p.name for p in package_root.glob("requirements*.txt"))
    errors = [f"unexpected requirements artifact: packages/atlasctl/{name}" for name in found if name not in _REQ_ALLOWED]
    if (package_root / "uv.lock").exists():
        errors.append("unexpected lock artifact for route-B workflow: packages/atlasctl/uv.lock")
    if not (package_root / "requirements.in").exists():
        errors.append("missing packages/atlasctl/requirements.in")
    if not (package_root / "requirements.lock.txt").exists():
        errors.append("missing packages/atlasctl/requirements.lock.txt")
    return (0 if not errors else 1), errors


def check_requirements_sync_with_pyproject(repo_root: Path) -> tuple[int, list[str]]:
    text = _read_pyproject(repo_root)
    block = re.search(r"(?ms)^dev\s*=\s*\[(.*?)\]", text)
    pyproject_dev = sorted(set(re.findall(r'"([^"]+)"', block.group(1) if block else "")))
    req_in = repo_root / "packages/atlasctl/requirements.in"
    req_lock = repo_root / "packages/atlasctl/requirements.lock.txt"
    req_lines = sorted(
        {
            ln.strip()
            for ln in req_in.read_text(encoding="utf-8").splitlines()
            if ln.strip() and not ln.strip().startswith("#")
        }
    )
    lock_lines = sorted(
        {
            ln.strip()
            for ln in req_lock.read_text(encoding="utf-8").splitlines()
            if ln.strip() and not ln.strip().startswith("#")
        }
    )
    errors: list[str] = []
    if pyproject_dev != req_lines:
        errors.append(f"requirements.in drift from pyproject optional-deps dev: pyproject={pyproject_dev} requirements.in={req_lines}")
    if req_lines != lock_lines:
        errors.append(f"requirements.lock.txt drift from requirements.in: in={req_lines} lock={lock_lines}")
    return (0 if not errors else 1), errors


def check_dependency_owner_justification(repo_root: Path) -> tuple[int, list[str]]:
    deps_doc = repo_root / "packages/atlasctl/docs/deps.md"
    text = deps_doc.read_text(encoding="utf-8")
    table_deps = set(re.findall(r"\|\s*`([^`]+==[^`]+)`\s*\|\s*`[^`]+`\s*\|\s*[^|]+\|", text))
    req_in = repo_root / "packages/atlasctl/requirements.in"
    req_deps = {
        ln.strip()
        for ln in req_in.read_text(encoding="utf-8").splitlines()
        if ln.strip() and not ln.strip().startswith("#")
    }
    missing = sorted(dep for dep in req_deps if dep not in table_deps)
    errors = [f"missing owner+justification in docs/deps.md for dependency: {dep}" for dep in missing]
    return (0 if not errors else 1), errors


def check_dependency_gate_targets(repo_root: Path) -> tuple[int, list[str]]:
    mk = (repo_root / "makefiles/scripts.mk").read_text(encoding="utf-8")
    required = ("deps-lock:", "deps-sync:", "deps-check-venv:", "deps-cold-start:")
    errors = [f"missing make dependency gate target: {name[:-1]}" for name in required if name not in mk]
    return (0 if not errors else 1), errors


def check_deps_command_surface(repo_root: Path) -> tuple[int, list[str]]:
    env = {"PYTHONPATH": str(repo_root / "packages/atlasctl/src")}
    proc = subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", "deps", "cold-start", "--runs", "1", "--max-ms", "5000"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )
    if proc.returncode == 0:
        return 0, []
    msg = proc.stderr.strip() or proc.stdout.strip() or "atlasctl deps command failed"
    return 1, [msg]
