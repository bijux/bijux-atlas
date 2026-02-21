from __future__ import annotations

import importlib
import re
from pathlib import Path


_REQUIRED_TOML_BLOCKS = (
    "[project]",
    "[project.scripts]",
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
