from __future__ import annotations

import ast
from pathlib import Path

_ALLOWED = {
    "packages/atlasctl/src/atlasctl/core/runtime/repo_root.py",
    "packages/atlasctl/src/atlasctl/checks/repo/enforcement/cwd_usage.py",
}
_PATH_DOT_DOUBLE = ('Path("', '.")')
_PATH_DOT_SINGLE = ("Path('", ".')")


def check_no_path_cwd_usage(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    relative_path_offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in _ALLOWED:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "Path.cwd(" in text:
            offenders.append(rel)
        if "/checks/" in rel or "/commands/" in rel:
            tree = ast.parse(text, filename=rel)
            for node in ast.walk(tree):
                if not isinstance(node, ast.Call):
                    continue
                if not isinstance(node.func, ast.Name) or node.func.id != "Path":
                    continue
                if not node.args or not isinstance(node.args[0], ast.Constant) or not isinstance(node.args[0].value, str):
                    continue
                value = node.args[0].value.strip()
                if not value or value.startswith("/") or value.startswith("~"):
                    continue
                line = (text.splitlines()[node.lineno - 1] if node.lineno - 1 < len(text.splitlines()) else "")
                if "repo_root" in line:
                    continue
                relative_path_offenders.append(f"{rel}:{node.lineno} uses relative Path('{value}') without repo_root anchoring")
    if offenders:
        return 1, ["Path.cwd() is forbidden outside core.runtime.repo_root.py", *offenders, *sorted(set(relative_path_offenders))]
    if relative_path_offenders:
        return 1, sorted(set(relative_path_offenders))
    return 0, []


def check_no_path_dot_in_runtime(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/checks/" not in rel and "/commands/check/" not in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if (_PATH_DOT_DOUBLE[0] + _PATH_DOT_DOUBLE[1]) in text or (_PATH_DOT_SINGLE[0] + _PATH_DOT_SINGLE[1]) in text:
            offenders.append(f"{rel}: forbidden dot-path usage; resolve through explicit repo_root")
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
