from __future__ import annotations

from pathlib import Path

from atlasctl.checks.domains.internal import (
    check_all_checks_declare_effects,
    check_all_checks_have_owner,
    check_all_checks_have_tags,
    check_checks_file_count_budget,
    check_checks_tree_depth_budget,
    check_domains_directory_shape,
    check_docs_checks_no_ops_imports,
    check_legacy_check_directories_absent,
    check_no_checks_outside_domains_tools,
    check_no_relative_imports_across_domains,
    check_no_tests_fixtures_imports,
    check_unused_imports_in_checks,
    check_internal_no_checks_logic_in_commands,
    check_internal_no_command_logic_in_checks,
    check_internal_registry_ssot_only,
    check_root_policy_compat_shims_not_expired,
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


def test_root_policy_compat_shims_require_expiry(tmp_path: Path) -> None:
    path = tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/root_policy.json"
    _write(
        path,
        """{
  "required": [],
  "allowed": [],
  "compat_shims": ["Dockerfile"],
  "local_noise": []
}
""",
    )
    code, errors = check_root_policy_compat_shims_not_expired(tmp_path)
    assert code == 1
    assert any("missing expiry" in line for line in errors)


def test_internal_command_checks_import_boundaries(tmp_path: Path) -> None:
    checks_file = tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/example.py"
    _write(checks_file, "from atlasctl.commands.check.command import run_check_command\n")
    code, errors = check_internal_no_command_logic_in_checks(tmp_path)
    assert code == 1
    assert errors


def test_internal_checks_commands_boundary(tmp_path: Path) -> None:
    commands_file = tmp_path / "packages/atlasctl/src/atlasctl/commands/demo.py"
    _write(commands_file, "from atlasctl.checks.domains.repo import CHECKS\n")
    code, errors = check_internal_no_checks_logic_in_commands(tmp_path)
    assert code == 1
    assert errors


def test_internal_registry_ssot_only_detects_scattered_legacy_import(tmp_path: Path) -> None:
    path = tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/example.py"
    _write(path, "from atlasctl.checks.registry_legacy.ssot import load_registry_entries\n")
    code, errors = check_internal_registry_ssot_only(tmp_path)
    assert code == 1
    assert any("registry legacy import must be isolated" in line for line in errors)


def test_docs_checks_do_not_import_ops_modules(tmp_path: Path) -> None:
    path = tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/docs_domain/example.py"
    _write(path, "from atlasctl.commands.ops.runtime import run\n")
    code, errors = check_docs_checks_no_ops_imports(tmp_path)
    assert code == 1
    assert any("docs checks must not import ops modules directly" in line for line in errors)


def test_checks_file_count_budget_flags_large_tree(tmp_path: Path) -> None:
    root = tmp_path / "packages/atlasctl/src/atlasctl/checks/tools"
    for idx in range(45):
        _write(root / f"f_{idx}.py", "X = 1\n")
    code, errors = check_checks_file_count_budget(tmp_path)
    assert code == 1
    assert any("file-count budget exceeded" in line for line in errors)


def test_checks_depth_budget_flags_deep_paths(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/a/b/c.py",
        "X = 1\n",
    )
    code, errors = check_checks_tree_depth_budget(tmp_path)
    assert code == 1
    assert any("depth exceeded" in line for line in errors)


def test_domains_directory_shape_flags_nested_directory(tmp_path: Path) -> None:
    _write(tmp_path / "packages/atlasctl/src/atlasctl/checks/domains/repo.py", "X = 1\n")
    (tmp_path / "packages/atlasctl/src/atlasctl/checks/domains/nested").mkdir(parents=True, exist_ok=True)
    code, errors = check_domains_directory_shape(tmp_path)
    assert code == 1
    assert any("domains root must contain only python modules" in line for line in errors)


def test_no_relative_imports_across_domains(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/domains/repo.py",
        "from .internal import CHECKS\n",
    )
    code, errors = check_no_relative_imports_across_domains(tmp_path)
    assert code == 1
    assert any("cross-domain relative imports are forbidden" in line for line in errors)


def test_no_tests_fixtures_imports_and_unused_imports(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/example.py",
        "from atlasctl.tests.fixtures import sample\nimport os\n\nX = 1\n",
    )
    code1, errors1 = check_no_tests_fixtures_imports(tmp_path)
    code2, errors2 = check_unused_imports_in_checks(tmp_path)
    assert code1 == 1
    assert code2 == 1
    assert errors1
    assert any("unused import" in line for line in errors2)
