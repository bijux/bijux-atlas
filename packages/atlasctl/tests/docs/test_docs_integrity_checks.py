from __future__ import annotations

from pathlib import Path

from atlasctl.checks.docs.integrity import (
    check_docs_index_complete,
    check_docs_links_exist,
    check_no_package_root_markdown_except_readme,
)


def test_docs_integrity_checks_pass_on_repo() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    for fn in (
        check_no_package_root_markdown_except_readme,
        check_docs_links_exist,
        check_docs_index_complete,
    ):
        code, errors = fn(repo_root)
        assert code == 0, (fn.__name__, errors)


def test_docs_links_exist_flags_missing_target(tmp_path: Path) -> None:
    (tmp_path / "packages/atlasctl/docs").mkdir(parents=True)
    (tmp_path / "packages/atlasctl/docs/index.md").write_text("[x](missing.md)\n", encoding="utf-8")
    code, errors = check_docs_links_exist(tmp_path)
    assert code == 1
    assert errors
