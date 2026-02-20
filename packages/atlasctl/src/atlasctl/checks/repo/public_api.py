from __future__ import annotations

import ast
import re
from pathlib import Path


def _extract_dunder_all_symbols(init_path: Path) -> set[str]:
    module = ast.parse(init_path.read_text(encoding="utf-8"))
    for node in module.body:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name) and target.id == "__all__":
                    if isinstance(node.value, (ast.List, ast.Tuple)):
                        values: set[str] = set()
                        for item in node.value.elts:
                            if isinstance(item, ast.Constant) and isinstance(item.value, str):
                                values.add(item.value)
                        return values
    return set()


def _extract_public_api_symbols(api_doc: Path) -> set[str]:
    text = api_doc.read_text(encoding="utf-8")
    found = set(re.findall(r"`atlasctl\.([A-Za-z_][A-Za-z0-9_]*)`", text))
    return found


def check_public_api_exports(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    init_path = repo_root / "packages/atlasctl/src/atlasctl/__init__.py"
    api_doc = repo_root / "packages/atlasctl/PUBLIC_API.md"
    if not init_path.exists():
        return 1, ["missing packages/atlasctl/src/atlasctl/__init__.py"]
    if not api_doc.exists():
        return 1, ["missing packages/atlasctl/PUBLIC_API.md"]

    exported = _extract_dunder_all_symbols(init_path)
    documented = _extract_public_api_symbols(api_doc)
    undocumented = sorted(name for name in exported if name not in documented)
    for name in undocumented:
        errors.append(f"__init__.__all__ symbol not documented in PUBLIC_API.md: {name}")
    return (0 if not errors else 1), errors
