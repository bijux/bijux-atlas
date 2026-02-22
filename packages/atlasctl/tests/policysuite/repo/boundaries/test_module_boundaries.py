from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.layout.boundary_check import check_boundaries

ROOT = Path(__file__).resolve().parents[3]


def test_module_boundary_check_passes_on_repo() -> None:
    violations = check_boundaries(ROOT)
    assert violations == []


def test_module_boundary_check_detects_forbidden_import(tmp_path: Path) -> None:
    pkg_root = tmp_path / "packages" / "atlasctl" / "src" / "atlasctl"
    (pkg_root / "ops").mkdir(parents=True)
    (pkg_root / "registry").mkdir(parents=True)
    (pkg_root / "ops" / "bad.py").write_text("import atlasctl.registry.pins\n", encoding="utf-8")
    (pkg_root / "registry" / "pins.py").write_text("x = 1\n", encoding="utf-8")

    violations = check_boundaries(tmp_path)
    assert len(violations) == 1
    assert violations[0].source == "ops"
    assert violations[0].target == "registry"
