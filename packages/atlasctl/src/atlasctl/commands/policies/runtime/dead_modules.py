from __future__ import annotations

import ast
import re
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
    "atlasctl.commands.check.command",
    "atlasctl.commands.configs.command",
}


def _iter_modules(src_root: Path) -> dict[str, Path]:
    modules: dict[str, Path] = {}
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(src_root).with_suffix("")
        name = ".".join(("atlasctl", *rel.parts))
        modules[name] = path
    return modules


_PY_PATH_RE = re.compile(r"(packages/atlasctl/src/atlasctl/[A-Za-z0-9_./-]+\.py)")
_MOD_REF_RE = re.compile(r"(atlasctl(?:\.[A-Za-z_][A-Za-z0-9_]*)+)")


def _collect_path_references(repo_root: Path) -> set[str]:
    refs: set[str] = set()
    scan_roots = [
        repo_root / "makefiles",
        repo_root / "packages" / "atlasctl" / "src" / "atlasctl" / "commands",
    ]
    for base in scan_roots:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix not in {".mk", ".py"}:
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            for match in _PY_PATH_RE.findall(text):
                refs.add(match)
    return refs


def _collect_module_references(repo_root: Path) -> set[str]:
    refs: set[str] = set()
    scan_roots = [repo_root / "packages" / "atlasctl" / "src" / "atlasctl"]
    for base in scan_roots:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*.py")):
            text = path.read_text(encoding="utf-8", errors="ignore")
            for match in _MOD_REF_RE.findall(text):
                refs.add(match)
    return refs


def _is_script_entrypoint(path: Path) -> bool:
    try:
        first = path.read_text(encoding="utf-8", errors="ignore").splitlines()[:1]
    except OSError:
        return False
    if not first:
        return False
    return first[0].startswith("#!")


def _resolve_import_from(module_name: str, node: ast.ImportFrom) -> str | None:
    if node.level == 0:
        return node.module

    base = module_name.split(".")
    base = base[:-node.level]
    if node.module:
        return ".".join([*base, node.module])
    return ".".join(base)


def analyze_dead_modules(repo_root: Path) -> dict[str, object]:
    src_root = repo_root / _SRC_ROOT
    module_map = _iter_modules(src_root)
    path_refs = _collect_path_references(repo_root)
    module_refs = _collect_module_references(repo_root)
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
        if module_name in module_refs:
            continue
        if "/commands/" in rel:
            continue
        if _is_script_entrypoint(path):
            continue
        if rel in path_refs:
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
