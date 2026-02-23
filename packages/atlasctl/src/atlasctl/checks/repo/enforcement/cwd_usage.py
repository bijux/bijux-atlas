from __future__ import annotations

import ast
from pathlib import Path

_ALLOWED = {
    "packages/atlasctl/src/atlasctl/core/runtime/repo_root.py",
    "packages/atlasctl/src/atlasctl/checks/repo/enforcement/cwd_usage.py",
}


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
