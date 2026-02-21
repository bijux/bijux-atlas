from __future__ import annotations

import subprocess
import sys
from pathlib import Path


_ALLOWED_SHELL_PREFIXES = (
    "packages/atlasctl/src/atlasctl/checks/layout/",
    "ops/",
)
_SHELL_HEADER = "#!/usr/bin/env bash"
_STRICT_MODE = "set -euo pipefail"


def _iter_shell_files(repo_root: Path) -> list[Path]:
    return sorted(p for p in repo_root.rglob("*.sh") if ".git/" not in p.as_posix())


def check_shell_location_policy(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in _iter_shell_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if rel.startswith(_ALLOWED_SHELL_PREFIXES):
            continue
        offenders.append(rel)
    if offenders:
        return 1, [
            "shell scripts are allowed only under packages/atlasctl/src/atlasctl/checks/layout/ and ops/",
            *offenders,
        ]
    return 0, []


def check_shell_headers_and_strict_mode(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in _iter_shell_files(repo_root):
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
    for path in _iter_shell_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), 1):
            striped = line.strip()
            if striped.startswith("#"):
                continue
            if any(token in striped for token in forbidden_tokens):
                offenders.append(f"{rel}:{lineno}: direct python invocation is forbidden in shell scripts")
                break
    return (0 if not offenders else 1), offenders


def check_shell_invocation_via_core_exec(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    allowed = {
        "packages/atlasctl/src/atlasctl/core/exec.py",
        "packages/atlasctl/src/atlasctl/checks/repo/enforcement/shell_policy.py",
    }
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in allowed or "/legacy/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if ".sh" not in text:
            continue
        if "subprocess.run(" in text or "subprocess.Popen(" in text or "os.system(" in text:
            offenders.append(f"{rel}: invoke shell scripts through core.exec helpers")
    return (0 if not offenders else 1), offenders


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

