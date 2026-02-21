from __future__ import annotations

import ast
from collections import defaultdict
from pathlib import Path


_SRC_ROOT = "packages/atlasctl/src/atlasctl"
_ROOT_MODULES = {
    "atlasctl",
    "atlasctl.__main__",
    "atlasctl.cli",
    "atlasctl.cli.main",
    "atlasctl.cli.surface_registry",
    "atlasctl.checks.registry",
    "atlasctl.commands.policies.runtime.command",
}


def _iter_modules(src_root: Path) -> dict[str, Path]:
    modules: dict[str, Path] = {}
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(src_root).with_suffix("")
        name = ".".join(("atlasctl", *rel.parts))
        modules[name] = path
    return modules


def _resolve_import_from(module_name: str, node: ast.ImportFrom) -> str | None:
    base = module_name.split(".")
    if node.level > 0:
        base = base[:-node.level]
    if node.module:
        return ".".join([*base, node.module])
    return ".".join(base)


def analyze_dead_modules(repo_root: Path) -> dict[str, object]:
    src_root = repo_root / _SRC_ROOT
    module_map = _iter_modules(src_root)
    inbound: dict[str, set[str]] = defaultdict(set)
    edges: dict[str, set[str]] = defaultdict(set)

    for module_name, path in module_map.items():
        text = path.read_text(encoding="utf-8", errors="ignore")
        try:
            tree = ast.parse(text, filename=path.as_posix())
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    target = alias.name
                    if target.startswith("atlasctl"):
                        edges[module_name].add(target)
            elif isinstance(node, ast.ImportFrom):
                target = _resolve_import_from(module_name, node)
                if target and target.startswith("atlasctl"):
                    edges[module_name].add(target)

    for source, targets in edges.items():
        for target in targets:
            if target in module_map:
                inbound[target].add(source)

    candidates: list[dict[str, object]] = []
    for module_name, path in sorted(module_map.items()):
        rel = path.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        if path.name == "__init__.py":
            continue
        if module_name in _ROOT_MODULES:
            continue
        incoming = sorted(inbound.get(module_name, set()))
        if incoming:
            continue
        candidates.append(
            {
                "module": module_name,
                "path": rel,
                "reason": "no inbound imports (potentially dead or dynamic-only)",
            }
        )

    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "total_modules": len(module_map),
        "candidate_count": len(candidates),
        "candidates": candidates,
    }
