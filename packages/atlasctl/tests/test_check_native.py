from __future__ import annotations

import json
from pathlib import Path

from atlasctl.check.native import (
    check_committed_generated_hygiene,
    check_duplicate_script_names,
    check_layout_contract,
    check_make_command_allowlist,
    check_make_forbidden_paths,
    check_make_scripts_references,
    check_no_xtask_refs,
    check_ops_generated_tracked,
    check_script_ownership,
    check_tracked_timestamp_paths,
)


def test_check_duplicate_script_names_detects_dash_underscore_conflict(tmp_path: Path) -> None:
    scripts = tmp_path / "scripts"
    scripts.mkdir()
    (scripts / "a_b.py").write_text("print('x')\n", encoding="utf-8")
    (scripts / "a-b.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    code, errors = check_duplicate_script_names(tmp_path)
    assert code == 1
    assert errors


def test_check_script_ownership_passes_for_mapped_paths(tmp_path: Path) -> None:
    meta = tmp_path / "configs/meta"
    meta.mkdir(parents=True)
    (tmp_path / "scripts/bin").mkdir(parents=True)
    (tmp_path / "scripts/bin/tool.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    ownership = {"paths": {"scripts/bin": "platform"}}
    (meta / "ownership.json").write_text(json.dumps(ownership), encoding="utf-8")
    code, errors = check_script_ownership(tmp_path)
    assert code == 0
    assert errors == []


def test_check_no_xtask_refs_flags_non_adr_mentions(tmp_path: Path) -> None:
    (tmp_path / "docs").mkdir()
    (tmp_path / "docs/page.md").write_text("use xtask command\n", encoding="utf-8")
    code, errors = check_no_xtask_refs(tmp_path)
    assert code == 1
    assert "docs/page.md" in errors


def test_check_make_forbidden_paths_blocks_tools_and_xtask(tmp_path: Path) -> None:
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "Makefile").write_text("all:\n\t@echo ok\n", encoding="utf-8")
    (tmp_path / "makefiles/test.mk").write_text("x:\n\t@python3 tools/foo.py\n", encoding="utf-8")
    code, errors = check_make_forbidden_paths(tmp_path)
    assert code == 1
    assert errors


def test_check_ops_generated_tracked_flags_tracked_entries(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.check.native._git_ls_files",
        lambda _repo_root, _spec: ["ops/_generated/run-1/report.json"],
    )
    code, errors = check_ops_generated_tracked(tmp_path)
    assert code == 1
    assert "ops/_generated" in errors[0]


def test_check_tracked_timestamp_paths_flags_timestamp_segments(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.check.native._git_ls_files",
        lambda _repo_root, _spec: ["artifacts/evidence/2026-02-20/report.json", "docs/index.md"],
    )
    code, errors = check_tracked_timestamp_paths(tmp_path)
    assert code == 1
    assert "2026-02-20" in errors[0]


def test_check_committed_generated_hygiene_flags_logs_and_timestamps(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "atlasctl.check.native._git_ls_files",
        lambda _repo_root, _spec: [
            "docs/_generated/2026-02-20/index.md",
            "ops/_generated_committed/run.log",
        ],
    )
    code, errors = check_committed_generated_hygiene(tmp_path)
    assert code == 1
    assert len(errors) == 2


def test_check_make_command_allowlist_passes_for_allowlisted_command(tmp_path: Path) -> None:
    (tmp_path / "configs/layout").mkdir(parents=True)
    (tmp_path / "configs/layout/make-command-allowlist.txt").write_text("echo\n", encoding="utf-8")
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "Makefile").write_text("all:\n\t@echo ok\n", encoding="utf-8")
    code, errors = check_make_command_allowlist(tmp_path)
    assert code == 0
    assert errors == []


def test_check_layout_contract_flags_unexpected_root_entry(tmp_path: Path) -> None:
    (tmp_path / "configs/repo").mkdir(parents=True)
    (tmp_path / ".github/workflows").mkdir(parents=True)
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "configs/repo/surfaces.json").write_text(
        json.dumps(
            {
                "allowed_root_dirs": [".github", "configs", "makefiles"],
                "allowed_root_files": ["README.md"],
            }
        ),
        encoding="utf-8",
    )
    (tmp_path / "configs/repo/root-files-allowlist.txt").write_text("README.md\n", encoding="utf-8")
    (tmp_path / "configs/repo/symlink-allowlist.json").write_text(
        json.dumps({"root": {}, "non_root": {}}),
        encoding="utf-8",
    )
    (tmp_path / ".github/workflows/ci.yml").write_text("- run: make check\n", encoding="utf-8")
    (tmp_path / "makefiles/root.mk").write_text("check:\n\t@echo ok\n", encoding="utf-8")
    (tmp_path / "README.md").write_text("x\n", encoding="utf-8")
    (tmp_path / "unexpected").mkdir()
    code, errors = check_layout_contract(tmp_path)
    assert code == 1
    assert any("unexpected root directory: unexpected" in e for e in errors)


def test_check_layout_contract_passes_minimal_repo(tmp_path: Path) -> None:
    (tmp_path / "configs/repo").mkdir(parents=True)
    (tmp_path / ".github/workflows").mkdir(parents=True)
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "configs/repo/surfaces.json").write_text(
        json.dumps(
            {
                "allowed_root_dirs": [".github", "configs", "makefiles"],
                "allowed_root_files": ["README.md"],
            }
        ),
        encoding="utf-8",
    )
    (tmp_path / "configs/repo/root-files-allowlist.txt").write_text("README.md\n", encoding="utf-8")
    (tmp_path / "configs/repo/symlink-allowlist.json").write_text(
        json.dumps({"root": {}, "non_root": {}}),
        encoding="utf-8",
    )
    (tmp_path / ".github/workflows/ci.yml").write_text("- run: make check\n", encoding="utf-8")
    (tmp_path / "makefiles/root.mk").write_text("check:\n\t@echo ok\n", encoding="utf-8")
    (tmp_path / "README.md").write_text("x\n", encoding="utf-8")
    code, errors = check_layout_contract(tmp_path)
    assert code == 0
    assert errors == []


def test_check_make_scripts_references_flags_unapproved_scripts_path(tmp_path: Path) -> None:
    (tmp_path / "configs/layout").mkdir(parents=True)
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "configs/layout/make-scripts-reference-exceptions.json").write_text(
        json.dumps({"exceptions": []}),
        encoding="utf-8",
    )
    (tmp_path / "Makefile").write_text("x:\n\t@./packages/atlasctl/src/atlasctl/layout_checks/check.sh\n", encoding="utf-8")
    code, errors = check_make_scripts_references(tmp_path)
    assert code == 1
    assert any("scripts/" in e for e in errors)
