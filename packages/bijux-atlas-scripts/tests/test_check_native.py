from __future__ import annotations

import json
from pathlib import Path

from bijux_atlas_scripts.check.native import (
    check_committed_generated_hygiene,
    check_duplicate_script_names,
    check_make_forbidden_paths,
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
    meta = tmp_path / "scripts/areas/_meta"
    meta.mkdir(parents=True)
    (tmp_path / "scripts/bin").mkdir(parents=True)
    (tmp_path / "scripts/bin/tool.sh").write_text("#!/usr/bin/env sh\n", encoding="utf-8")
    ownership = {"areas": ["scripts/bin", "scripts/areas/_meta"]}
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
        "bijux_atlas_scripts.check.native._git_ls_files",
        lambda _repo_root, _spec: ["ops/_generated/run-1/report.json"],
    )
    code, errors = check_ops_generated_tracked(tmp_path)
    assert code == 1
    assert "ops/_generated" in errors[0]


def test_check_tracked_timestamp_paths_flags_timestamp_segments(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "bijux_atlas_scripts.check.native._git_ls_files",
        lambda _repo_root, _spec: ["artifacts/evidence/2026-02-20/report.json", "docs/index.md"],
    )
    code, errors = check_tracked_timestamp_paths(tmp_path)
    assert code == 1
    assert "2026-02-20" in errors[0]


def test_check_committed_generated_hygiene_flags_logs_and_timestamps(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        "bijux_atlas_scripts.check.native._git_ls_files",
        lambda _repo_root, _spec: [
            "docs/_generated/2026-02-20/index.md",
            "ops/_generated_committed/run.log",
        ],
    )
    code, errors = check_committed_generated_hygiene(tmp_path)
    assert code == 1
    assert len(errors) == 2
