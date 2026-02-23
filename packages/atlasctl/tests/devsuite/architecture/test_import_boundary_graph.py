from __future__ import annotations

import ast
from pathlib import Path


SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
TOPS = {"app", "cli", "commands", "checks", "core", "runtime"}


def _module_name(py_file: Path) -> str:
    rel = py_file.relative_to(SRC_ROOT).with_suffix("")
    return ".".join(("atlasctl", *rel.parts))


def _top_import_edges() -> set[tuple[str, str]]:
    edges: set[tuple[str, str]] = set()
    for py_file in sorted(SRC_ROOT.rglob("*.py")):
        mod = _module_name(py_file)
        src_top = mod.split(".")[1] if mod.count(".") >= 1 else ""
        if src_top not in TOPS:
            continue
        tree = ast.parse(py_file.read_text(encoding="utf-8", errors="ignore"))
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                if node.module is None:
                    continue
                names = [node.module]
            else:
                continue
            for name in names:
                if not name.startswith("atlasctl."):
                    continue
                parts = name.split(".")
                if len(parts) < 2:
                    continue
                dst_top = parts[1]
                if dst_top not in TOPS:
                    continue
                edges.add((src_top, dst_top))
    return edges


def test_top_level_import_boundary_graph() -> None:
    edges = _top_import_edges()
    forbidden = {
        ("core", "commands"),
        ("core", "cli"),
        ("runtime", "commands"),
        ("runtime", "cli"),
        ("checks", "commands"),
    }
    legacy_allowlist = {
        # Temporary during Phase 3 cutover. Delete by 2026-04-01 with shim removals.
        ("runtime", "cli"),
        ("checks", "commands"),
    }
    violations = sorted(edge for edge in edges if edge in forbidden and edge not in legacy_allowlist)
    assert not violations, f"forbidden top-level import edges present: {violations}"
