from __future__ import annotations

import ast
from pathlib import Path

_ALLOWED = {
    "packages/atlasctl/src/atlasctl/core/runtime/repo_root.py",
    "packages/atlasctl/src/atlasctl/checks/tools/repo_domain/enforcement/cwd_usage.py",
    "packages/atlasctl/src/atlasctl/commands/check/command.py",
}


def check_no_path_cwd_usage(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in _ALLOWED:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "Path.cwd(" in text:
            offenders.append(rel)
    if offenders:
        return 1, ["Path.cwd() is forbidden outside core.runtime.repo_root.py", *sorted(set(offenders))]
    return 0, []


def check_no_path_dot_in_runtime(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/checks/" not in rel and "/commands/check/" not in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        try:
            tree = ast.parse(text, filename=rel)
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            if not isinstance(node.func, ast.Name) or node.func.id != "Path":
                continue
            if not node.args:
                continue
            first = node.args[0]
            if not isinstance(first, ast.Constant) or first.value != ".":
                continue
            offenders.append(f"{rel}:{getattr(node, 'lineno', 1)}: forbidden dot-path usage; resolve through explicit repo_root")
    return (0 if not offenders else 1), offenders


def check_no_environ_mutation(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/checks/" not in rel and "/commands/check/" not in rel:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if (
                        isinstance(target, ast.Subscript)
                        and isinstance(target.value, ast.Attribute)
                        and isinstance(target.value.value, ast.Name)
                        and target.value.value.id == "os"
                        and target.value.attr == "environ"
                    ):
                        offenders.append(f"{rel}:{node.lineno}: do not mutate os.environ in checks runtime")
            if isinstance(node, ast.Call):
                if (
                    isinstance(node.func, ast.Attribute)
                    and isinstance(node.func.value, ast.Attribute)
                    and isinstance(node.func.value.value, ast.Name)
                    and node.func.value.value.id == "os"
                    and node.func.value.attr == "environ"
                    and node.func.attr == "update"
                ):
                    offenders.append(f"{rel}:{node.lineno}: do not call os.environ.update in checks runtime")
                if (
                    isinstance(node.func, ast.Attribute)
                    and isinstance(node.func.value, ast.Name)
                    and node.func.value.id == "os"
                    and node.func.attr == "putenv"
                ):
                    offenders.append(f"{rel}:{node.lineno}: do not call os.putenv in checks runtime")
    return (0 if not offenders else 1), sorted(set(offenders))
