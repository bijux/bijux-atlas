from __future__ import annotations

import re
import sys
from pathlib import Path
import subprocess
import ast
import json


_VENDORED_SHELL_DIR = "ops/vendor/layout-checks"
_SHELL_HEADER = "#!/usr/bin/env bash"
_STRICT_MODE = "set -euo pipefail"
_MAX_SHELL_SCRIPTS = 16


def _read_allowlist(repo_root: Path, rel_path: str) -> set[str]:
    path = repo_root / rel_path
    if not path.exists():
        return set()
    if path.suffix == ".json":
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            return set()
        rows = payload.get("entries", []) if isinstance(payload, dict) else []
        values: set[str] = set()
        for row in rows:
            if isinstance(row, str):
                value = row.strip()
            elif isinstance(row, dict):
                value = str(row.get("path", "")).strip()
            else:
                value = ""
            if value:
                values.add(value)
        return values
    return {
        line.strip()
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }

def _iter_layout_shell_checks(repo_root: Path) -> list[Path]:
    root = repo_root / _VENDORED_SHELL_DIR
    return sorted(root.glob("*.sh"))


def check_shell_location_policy(repo_root: Path) -> tuple[int, list[str]]:
    offenders = [
        path.relative_to(repo_root).as_posix()
        for path in sorted((repo_root / "packages/atlasctl/src/atlasctl").rglob("*.sh"))
    ]
    if offenders:
        return 1, [
            "shell scripts are forbidden inside packages/atlasctl/src/atlasctl; quarantine under ops/vendor/layout-checks/ or port to python",
            *offenders,
        ]
    return 0, []


def check_shell_headers_and_strict_mode(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in _iter_layout_shell_checks(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        lines = text.splitlines()
        if not lines or lines[0].strip() != _SHELL_HEADER:
            offenders.append(f"{rel}: missing `{_SHELL_HEADER}` header")
            continue
        if _STRICT_MODE not in text:
            offenders.append(f"{rel}: missing `{_STRICT_MODE}`")
    return (0 if not offenders else 1), offenders


def check_shell_no_python_direct_calls(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    forbidden_tokens = ("python ", "python3 ", "python -m ", "python3 -m ")
    for path in _iter_layout_shell_checks(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), 1):
            striped = line.strip()
            if striped.startswith("#"):
                continue
            if any(token in striped for token in forbidden_tokens):
                offenders.append(f"{rel}:{lineno}: direct python invocation is forbidden in shell scripts")
                break
    return (0 if not offenders else 1), offenders


def check_shell_no_network_download_tools(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    allowed_paths = _read_allowlist(repo_root, "configs/policy/shell-network-fetch-allowlist.json")
    forbidden_tokens = ("curl ", "wget ")
    for path in _iter_layout_shell_checks(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if rel in allowed_paths:
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), 1):
            striped = line.strip()
            if striped.startswith("#"):
                continue
            if "shell-allow-network-fetch" in striped:
                continue
            if any(token in striped for token in forbidden_tokens):
                offenders.append(f"{rel}:{lineno}: direct curl/wget usage is forbidden")
                break
    return (0 if not offenders else 1), offenders


def check_shell_invocation_via_core_exec(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    allowed = {
        "packages/atlasctl/src/atlasctl/core/exec.py",
        "packages/atlasctl/src/atlasctl/core/exec_shell.py",
        "packages/atlasctl/src/atlasctl/checks/repo/enforcement/shell_policy.py",
    }
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in allowed or "/legacy/" in rel:
            continue
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            shell_invocation = False
            if isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name):
                is_subprocess = node.func.value.id == "subprocess" and node.func.attr in {"run", "Popen"}
                is_os_system = node.func.value.id == "os" and node.func.attr == "system"
                shell_invocation = is_subprocess or is_os_system
            if not shell_invocation or not node.args:
                continue
            arg0 = node.args[0]
            if isinstance(arg0, ast.Constant) and isinstance(arg0.value, str):
                if ".sh" in arg0.value or "bash" in arg0.value or "sh" in arg0.value:
                    offenders.append(f"{rel}:{getattr(node, 'lineno', 0)}: invoke shell scripts through core.exec helpers")
                    break
            if isinstance(arg0, ast.List):
                heads = [elt.value for elt in arg0.elts if isinstance(elt, ast.Constant) and isinstance(elt.value, str)]
                if any(head in {"bash", "sh"} for head in heads) or any(part.endswith(".sh") for part in heads):
                    offenders.append(f"{rel}:{getattr(node, 'lineno', 0)}: invoke shell scripts through core.exec helpers")
                    break
    return (0 if not offenders else 1), offenders


def check_core_no_bash_subprocess(repo_root: Path) -> tuple[int, list[str]]:
    allowlist = _read_allowlist(repo_root, "configs/policy/shell-probes-allowlist.json")
    core_root = repo_root / "packages/atlasctl/src/atlasctl/core"
    offenders: list[str] = []
    for path in sorted(core_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in allowlist:
            continue
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            func = node.func
            if not isinstance(func, ast.Attribute) or func.attr not in {"run", "Popen"}:
                continue
            if not isinstance(func.value, ast.Name) or func.value.id != "subprocess":
                continue
            if not node.args:
                continue
            first = node.args[0]
            if not isinstance(first, ast.List) or not first.elts:
                continue
            head = first.elts[0]
            if isinstance(head, ast.Constant) and isinstance(head.value, str) and head.value in {"bash", "sh"}:
                offenders.append(f"{rel}:{getattr(node, 'lineno', 0)}: subprocess {head.value} invocation forbidden in core logic")
                break
    return (0 if not offenders else 1), offenders


def check_shell_scripts_readonly(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    forbidden_markers = ("cp ", "mv ", "rm ", "touch ", "mkdir ")
    redirection = re.compile(r"(?:^|\s)(>>?|1>>?|2>>?)\s*(?![&/])")
    for path in _iter_layout_shell_checks(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), 1):
            striped = line.strip()
            if striped.startswith("#"):
                continue
            if "mktemp" in striped:
                continue
            if any(token in striped for token in forbidden_markers) or redirection.search(striped):
                offenders.append(f"{rel}:{lineno}: shell check scripts must not write files directly")
                break
    return (0 if not offenders else 1), offenders


def check_shell_script_budget(repo_root: Path) -> tuple[int, list[str]]:
    count = len(_iter_layout_shell_checks(repo_root))
    if count <= _MAX_SHELL_SCRIPTS:
        return 0, []
    return 1, [f"layout shell script budget exceeded: {count} > {_MAX_SHELL_SCRIPTS}"]


def check_shell_docs_present(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / _VENDORED_SHELL_DIR
    required = (
        root / "README.md",
        root / "POLICY.md",
    )
    errors = [f"missing shell docs file: {path.relative_to(repo_root).as_posix()}" for path in required if not path.exists()]
    return (0 if not errors else 1), errors


def check_no_layout_shadow_configs(repo_root: Path) -> tuple[int, list[str]]:
    script = repo_root / "packages/atlasctl/src/atlasctl/checks/repo/layout/no_shadow.py"
    proc = subprocess.run(
        [sys.executable, str(script)],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode == 0:
        return 0, []
    out = (proc.stdout or "").strip()
    err = (proc.stderr or "").strip()
    message = err or out or "layout no_shadow check failed"
    return 1, [message]
