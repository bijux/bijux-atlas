from __future__ import annotations

from pathlib import Path

from atlasctl.checks.docs.integrity import (
    check_command_group_docs_pages,
    check_docs_index_complete,
    check_docs_links_exist,
    check_migration_docs_not_stale,
    check_docs_registry_command_drift,
    check_no_package_root_markdown_except_readme,
    check_stable_command_examples_in_group_docs,
)


def test_docs_integrity_checks_pass_on_repo() -> None:
    repo_root = Path(__file__).resolve().parents[4]
    for fn in (
        check_no_package_root_markdown_except_readme,
        check_docs_links_exist,
        check_docs_index_complete,
        check_command_group_docs_pages,
        check_docs_registry_command_drift,
        check_stable_command_examples_in_group_docs,
        check_migration_docs_not_stale,
    ):
        code, errors = fn(repo_root)
        assert code == 0, (fn.__name__, errors)


def test_docs_links_exist_flags_missing_target(tmp_path: Path) -> None:
    (tmp_path / "packages/atlasctl/docs").mkdir(parents=True)
    (tmp_path / "packages/atlasctl/docs/index.md").write_text("[x](missing.md)\n", encoding="utf-8")
    code, errors = check_docs_links_exist(tmp_path)
    assert code == 1
    assert errors


def test_docs_registry_command_drift_flags_unknown_command(tmp_path: Path) -> None:
    docs = tmp_path / "packages/atlasctl/docs/commands"
    docs.mkdir(parents=True)
    (docs / "index.md").write_text("run `atlasctl not-a-real-command`\n", encoding="utf-8")
    code, errors = check_docs_registry_command_drift(tmp_path)
    assert code == 1
    assert any("not-a-real-command" in err for err in errors)
