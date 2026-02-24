from __future__ import annotations

from pathlib import Path

from atlasctl.checks.domains.internal import (
    check_checks_forbidden_imports,
    check_checks_import_cycles,
    check_registry_import_hygiene,
    check_scattered_registry_caches,
)


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def test_import_cycle_check_detects_cycle(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    _write(checks_root / "a.py", "from atlasctl.checks import b\n")
    _write(checks_root / "b.py", "from atlasctl.checks import a\n")

    code, errors = check_checks_import_cycles(tmp_path)
    assert code == 1
    assert any("import cycle in atlasctl.checks:" in line for line in errors)


def test_import_cycle_check_passes_without_cycle(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    _write(checks_root / "a.py", "from atlasctl.checks import shared\n")
    _write(checks_root / "shared.py", "VALUE = 1\n")

    code, errors = check_checks_import_cycles(tmp_path)
    assert code == 0
    assert errors == []


def test_forbidden_import_check_blocks_tests_and_fixtures(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    _write(checks_root / "a.py", "from atlasctl.commands.ops.tests import smoke\n")
    _write(checks_root / "b.py", "from atlasctl.ops.fixtures import sample\n")

    code, errors = check_checks_forbidden_imports(tmp_path)
    assert code == 1
    assert any("forbidden import" in line for line in errors)
    assert any("ops.tests" in line for line in errors)
    assert any("fixtures" in line for line in errors)


def test_forbidden_import_check_passes_for_runtime_imports(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    _write(checks_root / "a.py", "from atlasctl.checks.model import CheckDef\n")

    code, errors = check_checks_forbidden_imports(tmp_path)
    assert code == 0
    assert errors == []


def test_registry_import_hygiene_blocks_heavy_imports(tmp_path: Path) -> None:
    registry_root = tmp_path / "packages/atlasctl/src/atlasctl/checks/registry"
    _write(registry_root / "catalog.py", "import pandas as pd\n")

    code, errors = check_registry_import_hygiene(tmp_path)
    assert code == 1
    assert any("forbidden heavy import" in line for line in errors)


def test_registry_cache_scope_flags_scattered_lru_cache(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    _write(checks_root / "registry" / "catalog.py", "from functools import lru_cache\n@lru_cache(maxsize=1)\ndef idx():\n    return 1\n")
    _write(checks_root / "domains" / "x.py", "from functools import lru_cache\n@lru_cache(maxsize=1)\ndef nope():\n    return 2\n")

    code, errors = check_scattered_registry_caches(tmp_path)
    assert code == 1
    assert any("lru_cache usage outside registry index is forbidden" in line for line in errors)
