from __future__ import annotations

from pathlib import Path

from atlasctl.checks.domains.internal import (
    check_all_checks_declare_effects,
    check_all_checks_have_owner,
    check_all_checks_have_tags,
    check_legacy_check_directories_absent,
    check_no_checks_outside_domains_tools,
    check_registry_generated_read_only,
    check_write_roots_are_evidence_only,
)
from atlasctl.checks.model import CheckDef


def _ok(_repo_root: Path) -> tuple[int, list[str]]:
    return 0, []


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def test_checks_outside_domains_tools_fails_for_non_domain_module(monkeypatch, tmp_path: Path) -> None:
    check = CheckDef("checks_repo_sample_contract", "repo", "sample", 100, _ok, owners=("platform",))
    setattr(check.fn, "__module__", "atlasctl.checks.layout.sample")
    monkeypatch.setattr("atlasctl.checks.registry_legacy.catalog.list_checks", lambda: (check,))
    code, errors = check_no_checks_outside_domains_tools(tmp_path)
    assert code == 1
    assert errors


def test_legacy_directory_absence_detects_layout_and_repo(tmp_path: Path) -> None:
    (tmp_path / "packages/atlasctl/src/atlasctl/checks/layout").mkdir(parents=True)
    (tmp_path / "packages/atlasctl/src/atlasctl/checks/repo").mkdir(parents=True)
    code, errors = check_legacy_check_directories_absent(tmp_path)
    assert code == 1
    assert len(errors) == 2


def test_registry_generated_markers_required(tmp_path: Path) -> None:
    path = tmp_path / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json"
    _write(path, "{\"schema_version\":1,\"tool\":\"atlasctl\"}\n")
    code, errors = check_registry_generated_read_only(tmp_path)
    assert code == 0
    assert errors == []


def test_check_metadata_requirements(monkeypatch, tmp_path: Path) -> None:
    no_owner = CheckDef("checks_repo_a_contract", "repo", "a", 100, _ok, owners=())
    no_tags = CheckDef("checks_repo_b_contract", "repo", "b", 100, _ok, tags=(), owners=("platform",))
    no_effects = CheckDef("checks_repo_c_contract", "repo", "c", 100, _ok, effects=(), owners=("platform",))
    monkeypatch.setattr("atlasctl.checks.registry_legacy.catalog.list_checks", lambda: (no_owner, no_tags, no_effects))

    owner_code, owner_errors = check_all_checks_have_owner(tmp_path)
    tags_code, tags_errors = check_all_checks_have_tags(tmp_path)
    effects_code, effects_errors = check_all_checks_declare_effects(tmp_path)

    assert owner_code == 1 and owner_errors
    assert tags_code == 1 and tags_errors
    assert effects_code == 0  # CheckDef defaults to fs_read when not set
    assert effects_errors == []


def test_write_roots_restricted(monkeypatch, tmp_path: Path) -> None:
    bad = CheckDef(
        "checks_repo_write_contract",
        "repo",
        "write",
        100,
        _ok,
        effects=("fs_write",),
        writes_allowed_roots=("tmp/output",),
        owners=("platform",),
    )
    monkeypatch.setattr("atlasctl.checks.registry_legacy.catalog.list_checks", lambda: (bad,))
    code, errors = check_write_roots_are_evidence_only(tmp_path)
    assert code == 1
    assert any("managed evidence roots" in line for line in errors)
