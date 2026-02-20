from __future__ import annotations

import ast
import re
import sys
from pathlib import Path


TOOLING_DEPS = {"pytest", "pytest-timeout", "mypy", "ruff", "hypothesis"}
ALLOWED_UNDECLARED_IMPORTS = {"yaml", "tomllib", "schemas"}


def _top_imports(path: Path) -> set[str]:
    names: set[str] = set()
    try:
        module = ast.parse(path.read_text(encoding="utf-8"))
    except Exception:
        return names
    for node in ast.walk(module):
        if isinstance(node, ast.Import):
            for alias in node.names:
                names.add(alias.name.split(".")[0])
        elif isinstance(node, ast.ImportFrom):
            if node.module:
                names.add(node.module.split(".")[0])
    return names


def _third_party_imports(repo_root: Path) -> set[str]:
    names: set[str] = set()
    atlas_root = repo_root / "packages/atlasctl/src/atlasctl"
    local_modules = {p.stem for p in atlas_root.rglob("*.py")}
    local_modules.update({p.name for p in atlas_root.iterdir() if p.is_dir()})
    for path in sorted((repo_root / "packages/atlasctl/src").rglob("*.py")):
        for name in _top_imports(path):
            if name in {"atlasctl", "__future__"}:
                continue
            if name in local_modules:
                continue
            if name in sys.stdlib_module_names:
                continue
            names.add(name)
    return names


def _normalize_dist(name: str) -> str:
    return name.split("[")[0].split("==")[0].split(">=")[0].split("<=")[0].strip()


def check_dependency_declarations(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    deps_match = re.search(r"dependencies\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    dev_match = re.search(r"\[project\.optional-dependencies\]\s*dev\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    deps = {_normalize_dist(x) for x in re.findall(r'"([^"]+)"', deps_match.group("body"))} if deps_match else set()
    dev = {_normalize_dist(x) for x in re.findall(r'"([^"]+)"', dev_match.group("body"))} if dev_match else set()
    declared = deps | dev
    imported = _third_party_imports(repo_root)
    missing = sorted(name for name in imported if name not in declared and name not in ALLOWED_UNDECLARED_IMPORTS)
    if missing:
        errors.append(f"undeclared third-party imports: {', '.join(missing)}")
    unused = sorted(name for name in declared if name not in imported and name not in TOOLING_DEPS)
    if unused:
        errors.append(f"declared but unused dependencies: {', '.join(unused)}")
    return (0 if not errors else 1), errors
