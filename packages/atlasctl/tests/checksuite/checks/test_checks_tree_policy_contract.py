from __future__ import annotations

from pathlib import Path

from atlasctl.checks.domains.internal import (
    check_internal_checks_root_budget,
    check_internal_checks_tree_policy,
    check_internal_no_generated_registry_as_input,
    check_internal_no_registry_toml_reads,
)


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def test_tree_policy_flags_legacy_directories(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    (checks_root / "layout").mkdir(parents=True, exist_ok=True)
    (checks_root / "repo").mkdir(parents=True, exist_ok=True)
    (checks_root / "registry").mkdir(parents=True, exist_ok=True)
    code, errors = check_internal_checks_tree_policy(tmp_path)
    assert code == 1
    assert len(errors) >= 3


def test_root_budget_flags_overflow(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    checks_root.mkdir(parents=True, exist_ok=True)
    for i in range(16):
        _write(checks_root / f"f{i}.py", "x=1\n")
    code, errors = check_internal_checks_root_budget(tmp_path)
    assert code == 1
    assert any("budget exceeded" in line for line in errors)


def test_registry_artifact_reads_flagged_outside_generation_modules(tmp_path: Path) -> None:
    checks_root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    _write(checks_root / "domains" / "x.py", "DATA='REGISTRY.toml'\n")
    _write(checks_root / "domains" / "y.py", "DATA='REGISTRY.generated.json'\n")
    toml_code, toml_errors = check_internal_no_registry_toml_reads(tmp_path)
    generated_code, generated_errors = check_internal_no_generated_registry_as_input(tmp_path)
    assert toml_code == 1 and toml_errors
    assert generated_code == 1 and generated_errors
