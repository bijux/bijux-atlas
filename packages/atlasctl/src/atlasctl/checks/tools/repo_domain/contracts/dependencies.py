from __future__ import annotations

import ast
import json
import re
import sys
from pathlib import Path


TOOLING_DEPS = {"pytest", "pytest-timeout", "mypy", "ruff", "hypothesis"}
ALLOWED_UNDECLARED_IMPORTS = {"yaml", "tomllib", "schemas"}
_EXCEPTIONS_PATH = Path("configs/policy/dependency-exceptions.json")


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
            if node.level == 0 and node.module:
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


def _third_party_imports_by_file(repo_root: Path) -> dict[str, set[str]]:
    result: dict[str, set[str]] = {}
    atlas_root = repo_root / "packages/atlasctl/src/atlasctl"
    local_modules = {p.stem for p in atlas_root.rglob("*.py")}
    local_modules.update({p.name for p in atlas_root.iterdir() if p.is_dir()})
    for path in sorted((repo_root / "packages/atlasctl/src").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        deps: set[str] = set()
        for name in _top_imports(path):
            if name in {"atlasctl", "__future__"}:
                continue
            if name in local_modules:
                continue
            if name in sys.stdlib_module_names:
                continue
            deps.add(name)
        if deps:
            result[rel] = deps
    return result


def _normalize_dist(name: str) -> str:
    return name.split("[")[0].split("==")[0].split(">=")[0].split("<=")[0].strip()


def _load_dependency_exceptions(repo_root: Path) -> tuple[dict, list[str]]:
    path = repo_root / _EXCEPTIONS_PATH
    if not path.exists():
        return {}, [f"missing dependency exceptions file: {_EXCEPTIONS_PATH.as_posix()}"]
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return {}, [f"invalid dependency exceptions json: {exc}"]
    errors: list[str] = []
    for key in ("undeclared_import_allowlist", "optional_dependency_usage_allowlist", "internal_third_party_allowlist"):
        val = data.get(key)
        if not isinstance(val, list):
            errors.append(f"{_EXCEPTIONS_PATH.as_posix()} key `{key}` must be a list")
    for key in ("undeclared_import_allowlist", "optional_dependency_usage_allowlist", "internal_third_party_allowlist"):
        for idx, row in enumerate(data.get(key, [])):
            if not isinstance(row, dict):
                errors.append(f"{_EXCEPTIONS_PATH.as_posix()} `{key}`[{idx}] must be an object")
                continue
            if not str(row.get("justification", "")).strip():
                errors.append(f"{_EXCEPTIONS_PATH.as_posix()} `{key}`[{idx}] missing non-empty `justification`")
    return data, errors


def check_dependency_declarations(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    exceptions, exception_errors = _load_dependency_exceptions(repo_root)
    errors.extend(exception_errors)
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    deps_match = re.search(r"dependencies\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    dev_match = re.search(r"\[project\.optional-dependencies\]\s*dev\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    deps = {_normalize_dist(x) for x in re.findall(r'"([^"]+)"', deps_match.group("body"))} if deps_match else set()
    dev = {_normalize_dist(x) for x in re.findall(r'"([^"]+)"', dev_match.group("body"))} if dev_match else set()
    declared = deps | dev
    imported = _third_party_imports(repo_root)
    allowed_undeclared = {
        str(item.get("dependency", "")).strip()
        for item in exceptions.get("undeclared_import_allowlist", [])
        if isinstance(item, dict)
    }
    missing = sorted(name for name in imported if name not in declared and name not in ALLOWED_UNDECLARED_IMPORTS and name not in allowed_undeclared)
    if missing:
        errors.append(f"undeclared third-party imports: {', '.join(missing)}")
    unused = sorted(name for name in declared if name not in imported and name not in TOOLING_DEPS)
    if unused:
        errors.append(f"declared but unused dependencies: {', '.join(unused)}")
    return (0 if not errors else 1), errors


def _load_declared_dependencies(repo_root: Path) -> tuple[set[str], dict[str, set[str]]]:
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    deps_match = re.search(r"dependencies\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    deps = {_normalize_dist(x) for x in re.findall(r'"([^"]+)"', deps_match.group("body"))} if deps_match else set()
    opt: dict[str, set[str]] = {}
    block = re.search(r"(?ms)^\[project\.optional-dependencies\]\n(.*?)(?:^\[|\Z)", text)
    body = block.group(1) if block else ""
    for name in re.findall(r"(?m)^([A-Za-z0-9_-]+)\s*=", body):
        m = re.search(rf'(?ms)^{re.escape(name)}\s*=\s*\[(.*?)\]\s*(?:^[A-Za-z0-9_-]+\s*=|\Z)', body)
        vals = {_normalize_dist(x) for x in re.findall(r'"([^"]+)"', m.group(1) if m else "")}
        opt[name] = vals
    return deps, opt


def check_optional_dependency_usage_gates(repo_root: Path) -> tuple[int, list[str]]:
    deps, optional = _load_declared_dependencies(repo_root)
    file_imports = _third_party_imports_by_file(repo_root)
    exceptions, errors = _load_dependency_exceptions(repo_root)
    allow_pairs = {
        (str(item.get("path", "")).strip(), str(item.get("dependency", "")).strip())
        for item in exceptions.get("optional_dependency_usage_allowlist", [])
        if isinstance(item, dict)
    }
    optional_all = set().union(*optional.values()) if optional else set()
    offenders: list[str] = []
    for rel, imports in sorted(file_imports.items()):
        if "/tests/" in rel:
            continue
        for dep in sorted(imports):
            if dep in deps:
                continue
            if dep in optional_all and (rel, dep) not in allow_pairs:
                offenders.append(
                    f"{rel}: optional dependency `{dep}` used without allowlist gate in {_EXCEPTIONS_PATH.as_posix()}",
                )
    return (0 if not offenders and not errors else 1), [*errors, *offenders]


def check_internal_utils_stdlib_only(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl/internal"
    if not root.exists():
        return 0, []
    exceptions, errors = _load_dependency_exceptions(repo_root)
    allow = {
        str(item.get("dependency", "")).strip()
        for item in exceptions.get("internal_third_party_allowlist", [])
        if isinstance(item, dict)
    }
    offenders: list[str] = []
    atlas_root = repo_root / "packages/atlasctl/src/atlasctl"
    local_modules = {p.stem for p in atlas_root.rglob("*.py")}
    local_modules.update({p.name for p in atlas_root.iterdir() if p.is_dir()})
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        for name in sorted(_top_imports(path)):
            if name in {"atlasctl", "__future__"} or name in local_modules or name in sys.stdlib_module_names:
                continue
            if name in allow:
                continue
            offenders.append(f"{rel}: internal utilities should be stdlib-only; found third-party import `{name}`")
    return (0 if not offenders and not errors else 1), [*errors, *offenders]
