from __future__ import annotations

import re
import sys
from pathlib import Path
import subprocess


_ALLOWED_SHELL_PREFIXES = (
    "packages/atlasctl/src/atlasctl/checks/layout/shell/",
)
_SHELL_HEADER = "#!/usr/bin/env bash"
_STRICT_MODE = "set -euo pipefail"
_MAX_SHELL_SCRIPTS = 16

def _iter_layout_shell_checks(repo_root: Path) -> list[Path]:
    root = repo_root / "packages/atlasctl/src/atlasctl/checks/layout/shell"
    return sorted(root.glob("*.sh"))


def check_shell_location_policy(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / "packages/atlasctl/src/atlasctl/checks/layout").rglob("*.sh")):
        rel = path.relative_to(repo_root).as_posix()
        if rel.startswith(_ALLOWED_SHELL_PREFIXES):
            continue
        offenders.append(rel)
    if offenders:
        return 1, [
            "layout shell scripts are allowed only under packages/atlasctl/src/atlasctl/checks/layout/shell/",
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
    forbidden_tokens = ("curl ", "wget ")
    for path in _iter_layout_shell_checks(repo_root):
        rel = path.relative_to(repo_root).as_posix()
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
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        for idx, line in enumerate(lines):
            snippet = " ".join(lines[idx : idx + 3])
            if not any(token in snippet for token in ("subprocess.run(", "subprocess.Popen(", "os.system(")):
                continue
            if not any(token in snippet for token in (".sh", "\"bash\"", "'bash'", "\"sh\"", "'sh'")):
                continue
            offenders.append(f"{rel}: invoke shell scripts through core.exec helpers")
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
    root = repo_root / "packages/atlasctl/src/atlasctl/checks/layout/shell"
    required = (
        root / "README.md",
        root / "POLICY.md",
    )
    errors = [f"missing shell docs file: {path.relative_to(repo_root).as_posix()}" for path in required if not path.exists()]
    return (0 if not errors else 1), errors


def check_no_layout_shadow_configs(repo_root: Path) -> tuple[int, list[str]]:
    script = repo_root / "packages/atlasctl/src/atlasctl/layout/no_shadow.py"
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
