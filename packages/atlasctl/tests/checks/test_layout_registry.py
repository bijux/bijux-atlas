from __future__ import annotations

import ast
from pathlib import Path

import atlasctl
from atlasctl.checks.layout_catalog import list_layout_checks


SRC_ROOT = Path(atlasctl.__file__).resolve().parents[1]
LAYOUT_ROOT = SRC_ROOT / "atlasctl/checks/layout"


def _discover_layout_check_modules() -> set[str]:
    modules: set[str] = set()
    for path in sorted(LAYOUT_ROOT.rglob("check_*.py")):
        source = path.read_text(encoding="utf-8")
        tree = ast.parse(source)
        has_check_id = any(
            isinstance(node, ast.Assign)
            and any(isinstance(target, ast.Name) and target.id == "CHECK_ID" for target in node.targets)
            for node in tree.body
        )
        has_run = any(isinstance(node, ast.FunctionDef) and node.name == "run" for node in tree.body)
        if not (has_check_id and has_run):
            continue
        rel = path.relative_to(SRC_ROOT).with_suffix("")
        module_name = ".".join(rel.parts)
        modules.add(module_name)
    return modules


def test_layout_registry_covers_all_layout_check_modules() -> None:
    discovered = _discover_layout_check_modules()
    registered = {spec.module for spec in list_layout_checks()}
    assert registered == discovered


def test_layout_registry_ids_are_unique_and_file_aligned() -> None:
    specs = list_layout_checks()
    ids = [spec.check_id for spec in specs]
    assert len(ids) == len(set(ids))
    for spec in specs:
        filename = spec.module.rsplit(".", 1)[-1]
        expected_suffix = filename.removeprefix("check_")
        assert expected_suffix in spec.check_id
