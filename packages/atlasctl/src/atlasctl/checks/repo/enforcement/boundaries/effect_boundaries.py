from __future__ import annotations

import ast
import json
from pathlib import Path

_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_EXCEPTIONS_PATH = Path("configs/policy/effect-boundary-exceptions.json")
_CORE_EXEC = "packages/atlasctl/src/atlasctl/core/exec.py"
_CORE_PROCESS = "packages/atlasctl/src/atlasctl/core/process.py"
_CORE_FS = "packages/atlasctl/src/atlasctl/core/fs.py"
_CORE_ENV = "packages/atlasctl/src/atlasctl/core/env.py"
_CORE_NETWORK = "packages/atlasctl/src/atlasctl/core/network.py"
_RULES = (
    "subprocess_import",
    "subprocess_call",
    "path_write_call",
    "open_write_call",
    "os_environ_access",
    "network_call",
)


def _iter_modern_py(repo_root: Path) -> list[Path]:
    root = repo_root / _SRC_ROOT
    return sorted(path for path in root.rglob("*.py") if "/legacy/" not in path.as_posix())


def _is_core_path(rel: str, core_path: str) -> bool:
    return rel == core_path


def _load_exceptions(repo_root: Path) -> tuple[dict[str, set[str]], list[str]]:
    errors: list[str] = []
    configured: dict[str, set[str]] = {rule: set() for rule in _RULES}
    path = repo_root / _EXCEPTIONS_PATH
    if not path.exists():
        return configured, [f"{_EXCEPTIONS_PATH.as_posix()}: missing effect boundary exceptions config"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    rules = payload.get("rules", {})
    if not isinstance(rules, dict):
        return configured, [f"{_EXCEPTIONS_PATH.as_posix()}: rules must be an object"]
    for rule in _RULES:
        entries = rules.get(rule, [])
        if not isinstance(entries, list):
            errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: rules.{rule} must be a list")
            continue
        for idx, item in enumerate(entries):
            if not isinstance(item, dict):
                errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: rules.{rule}[{idx}] must be an object")
                continue
            rel_path = item.get("path", "")
            reason = item.get("reason", "")
            if not isinstance(rel_path, str) or not rel_path:
                errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: rules.{rule}[{idx}] path must be non-empty")
                continue
            if not isinstance(reason, str) or not reason.strip():
                errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: rules.{rule}[{idx}] reason must be non-empty")
            configured[rule].add(rel_path)
    return configured, errors


def _open_mode_is_mutating(call: ast.Call) -> bool:
    if len(call.args) >= 2 and isinstance(call.args[1], ast.Constant) and isinstance(call.args[1].value, str):
        mode = call.args[1].value
        return any(flag in mode for flag in ("w", "a", "+"))
    for kw in call.keywords:
        if kw.arg == "mode" and isinstance(kw.value, ast.Constant) and isinstance(kw.value.value, str):
            mode = kw.value.value
            return any(flag in mode for flag in ("w", "a", "+"))
    return False


def _rule_violations_for_file(path: Path, rel: str) -> set[str]:
    text = path.read_text(encoding="utf-8", errors="ignore")
    tree = ast.parse(text, filename=rel)
    violations: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.name == "subprocess":
                    violations.add("subprocess_import")
                if alias.name == "requests" or alias.name.startswith("urllib"):
                    violations.add("network_call")
        elif isinstance(node, ast.ImportFrom):
            if node.module == "subprocess":
                violations.add("subprocess_import")
            if node.module and (node.module == "requests" or node.module.startswith("urllib")):
                violations.add("network_call")
        elif isinstance(node, ast.Call):
            if isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name):
                if node.func.value.id == "subprocess":
                    violations.add("subprocess_call")
                if node.func.attr in {"write_text", "write_bytes"}:
                    violations.add("path_write_call")
                if node.func.value.id == "os" and node.func.attr == "environ":
                    violations.add("os_environ_access")
            if isinstance(node.func, ast.Name):
                if node.func.id == "open" and _open_mode_is_mutating(node):
                    violations.add("open_write_call")
                if node.func.id in {"urlopen", "Request"}:
                    violations.add("network_call")
            if isinstance(node.func, ast.Attribute):
                if isinstance(node.func.value, ast.Name) and node.func.value.id == "urllib":
                    violations.add("network_call")
                if node.func.attr in {"get", "post", "put", "patch", "delete", "request"}:
                    if isinstance(node.func.value, ast.Name) and node.func.value.id == "requests":
                        violations.add("network_call")
        elif isinstance(node, ast.Subscript):
            if isinstance(node.value, ast.Attribute) and isinstance(node.value.value, ast.Name):
                if node.value.value.id == "os" and node.value.attr == "environ":
                    violations.add("os_environ_access")
    return violations


def _scan_effect_boundary_violations(repo_root: Path) -> tuple[dict[str, list[str]], list[str]]:
    exceptions, exception_errors = _load_exceptions(repo_root)
    found: dict[str, set[str]] = {rule: set() for rule in _RULES}
    unknown_exceptions: list[str] = []
    for path in _iter_modern_py(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        violations = _rule_violations_for_file(path, rel)
        for rule in sorted(violations):
            if rule.startswith("subprocess") and (_is_core_path(rel, _CORE_EXEC) or _is_core_path(rel, _CORE_PROCESS)):
                continue
            if rule in {"path_write_call", "open_write_call"} and _is_core_path(rel, _CORE_FS):
                continue
            if rule == "os_environ_access" and _is_core_path(rel, _CORE_ENV):
                continue
            if rule == "network_call" and _is_core_path(rel, _CORE_NETWORK):
                continue
            found[rule].add(rel)
            if rel not in exceptions[rule]:
                unknown_exceptions.append(f"{rel}: {rule} outside enforced core boundary")
    stale: list[str] = []
    for rule, allowed in exceptions.items():
        for rel in sorted(allowed):
            if rel not in found[rule]:
                stale.append(f"{_EXCEPTIONS_PATH.as_posix()}: stale exception {rule} -> {rel}")
    combined_errors = sorted(set(exception_errors + unknown_exceptions + stale))
    normalized = {rule: sorted(paths) for rule, paths in found.items()}
    return normalized, combined_errors


def check_forbidden_effect_calls(repo_root: Path) -> tuple[int, list[str]]:
    _, errors = _scan_effect_boundary_violations(repo_root)
    return (0 if not errors else 1), errors


def check_subprocess_boundary(repo_root: Path) -> tuple[int, list[str]]:
    _, errors = _scan_effect_boundary_violations(repo_root)
    filtered = [err for err in errors if ": subprocess_" in err or _EXCEPTIONS_PATH.as_posix() in err]
    return (0 if not filtered else 1), sorted(set(filtered))


def check_effect_boundary_exceptions_policy(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _EXCEPTIONS_PATH
    if not path.exists():
        return 1, [f"{_EXCEPTIONS_PATH.as_posix()}: missing"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    rules = payload.get("rules", {})
    errors: list[str] = []
    for rule in _RULES:
        entries = rules.get(rule, [])
        if not isinstance(entries, list):
            errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: rules.{rule} must be a list")
            continue
        seen: set[str] = set()
        paths = [entry.get("path", "") for entry in entries if isinstance(entry, dict)]
        if paths != sorted(paths):
            errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: rules.{rule} paths must be sorted")
        for entry in entries:
            if not isinstance(entry, dict):
                continue
            rel = entry.get("path", "")
            reason = entry.get("reason", "")
            if rel in seen:
                errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: duplicate {rule} path {rel}")
            seen.add(rel)
            if not reason:
                errors.append(f"{_EXCEPTIONS_PATH.as_posix()}: {rule} path {rel} missing reason")
    return (0 if not errors else 1), sorted(set(errors))
