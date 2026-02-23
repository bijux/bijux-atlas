from __future__ import annotations

import re
from pathlib import Path

from ....core.process import run_command

_TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):")
_WORKFLOW_MAKE_RE = re.compile(r"\bmake(?:\s+-[A-Za-z0-9_-]+)*\s+([A-Za-z0-9_./-]+)")
_WORKFLOW_RUN_RE = re.compile(r"^\s*run:\s*(.+?)\s*$")
_RAW_CARGO_RE = re.compile(r"\bcargo\s+(fmt|test|clippy|check)\b")
_ATLASCTL_MODULE_RE = re.compile(r"\bpython3?\s+-m\s+atlasctl(\.cli)?\b")
_COMPILEALL_RE = re.compile(r"\bpython3?\s+-m\s+compileall\b")
_ALLOWED_ARTIFACT_ROOTS = ("artifacts/**", "ops/_generated/**", "ops/_generated.example/**")


def _make_targets(repo_root: Path) -> set[str]:
    result = run_command(["make", "-qp"], cwd=repo_root)
    targets: set[str] = set()
    for line in result.stdout.splitlines():
        match = _TARGET_RE.match(line)
        if not match:
            continue
        name = match.group(1).strip()
        if not name or name.startswith(".") or "%" in name:
            continue
        targets.add(name)
    return targets


def check_workflows_targets_exist(repo_root: Path) -> tuple[int, list[str]]:
    known_targets = _make_targets(repo_root)
    if not known_targets:
        return 1, ["failed to enumerate make targets via `make -qp`"]
    errors: list[str] = []
    workflows_root = repo_root / ".github" / "workflows"
    for workflow in sorted(workflows_root.glob("*.yml")):
        text = workflow.read_text(encoding="utf-8", errors="ignore")
        for line_no, line in enumerate(text.splitlines(), start=1):
            for match in _WORKFLOW_MAKE_RE.finditer(line):
                target = match.group(1)
                if target not in known_targets:
                    errors.append(f"{workflow.relative_to(repo_root).as_posix()}:{line_no}: missing make target `{target}`")
    return (0 if not errors else 1), sorted(errors)


def check_ci_workflow_policy(repo_root: Path) -> tuple[int, list[str]]:
    known_targets = _make_targets(repo_root)
    errors: list[str] = []
    workflows_root = repo_root / ".github" / "workflows"
    for workflow in sorted(workflows_root.glob("*.yml")):
        rel = workflow.relative_to(repo_root).as_posix()
        text = workflow.read_text(encoding="utf-8", errors="ignore")
        if workflow.name == "ci.yml" and "PYTHONDONTWRITEBYTECODE" not in text:
            errors.append(f"{rel}: missing PYTHONDONTWRITEBYTECODE guard for atlasctl CI test steps")
        for line_no, line in enumerate(text.splitlines(), start=1):
            run_match = _WORKFLOW_RUN_RE.match(line)
            if not run_match:
                continue
            cmd = run_match.group(1).strip()
            if _ATLASCTL_MODULE_RE.search(cmd):
                errors.append(f"{rel}:{line_no}: workflow must not invoke atlasctl via `python -m`; use `./bin/atlasctl`")
            if _RAW_CARGO_RE.search(cmd):
                errors.append(f"{rel}:{line_no}: workflow must not run raw cargo fmt/test/clippy/check")
            if _COMPILEALL_RE.search(cmd):
                errors.append(f"{rel}:{line_no}: workflow must not run `python -m compileall` (writes bytecode into source tree)")
            for make_match in _WORKFLOW_MAKE_RE.finditer(cmd):
                target = make_match.group(1)
                if target not in known_targets:
                    errors.append(f"{rel}:{line_no}: workflow references missing make target `{target}`")
                if target.startswith("internal/"):
                    errors.append(f"{rel}:{line_no}: workflow must not invoke internal make target `{target}`")
    return (0 if not errors else 1), sorted(errors)


def check_ci_artifact_upload_policy(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    workflows_root = repo_root / ".github" / "workflows"
    for workflow in sorted(workflows_root.glob("*.yml")):
        rel = workflow.relative_to(repo_root).as_posix()
        lines = workflow.read_text(encoding="utf-8", errors="ignore").splitlines()
        i = 0
        while i < len(lines):
            line = lines[i]
            if "uses: actions/upload-artifact" not in line:
                i += 1
                continue
            step_indent = len(line) - len(line.lstrip(" "))
            has_always = False
            probe = i
            while probe > 0:
                prev = lines[probe - 1]
                prev_indent = len(prev) - len(prev.lstrip(" "))
                if prev.lstrip(" ").startswith("- ") and prev_indent <= step_indent:
                    break
                if prev.strip().startswith("if:") and "always()" in prev.strip():
                    has_always = True
                    break
                probe -= 1
            probe = i
            while probe < len(lines):
                raw = lines[probe]
                if probe > i and raw.lstrip(" ").startswith("- ") and (len(raw) - len(raw.lstrip(" "))) <= step_indent:
                    break
                stripped = raw.strip()
                if stripped.startswith("if:") and "always()" in stripped:
                    has_always = True
                    break
                probe += 1
            if not has_always:
                errors.append(f"{rel}:{i+1}: artifact upload must use `if: always()`")
            k = i + 1
            while k < len(lines) and "path:" not in lines[k]:
                k += 1
            if k >= len(lines):
                errors.append(f"{rel}:{i+1}: upload-artifact step missing `path`")
                i += 1
                continue
            path_line = lines[k].strip()
            paths: list[str] = []
            if path_line.startswith("path: |"):
                base_indent = len(lines[k]) - len(lines[k].lstrip(" "))
                k += 1
                while k < len(lines):
                    raw = lines[k]
                    if raw.strip() == "":
                        k += 1
                        continue
                    indent = len(raw) - len(raw.lstrip(" "))
                    if indent <= base_indent:
                        break
                    paths.append(raw.strip())
                    k += 1
            else:
                _, value = path_line.split(":", 1)
                if value.strip():
                    paths.append(value.strip())
            for p in paths:
                if not any(p == allowed or p.startswith(allowed.rstrip("*")) for allowed in _ALLOWED_ARTIFACT_ROOTS):
                    errors.append(f"{rel}:{k+1}: artifact path `{p}` is outside allowed roots {list(_ALLOWED_ARTIFACT_ROOTS)}")
            i = k + 1
    return (0 if not errors else 1), sorted(errors)
