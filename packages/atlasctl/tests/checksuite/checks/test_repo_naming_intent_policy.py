from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.repo_domain.native_lint import check_naming_intent_lint


def _mk_repo(tmp_path: Path) -> Path:
    for root in ("crates", "packages", "ops", "configs", "makefiles"):
        (tmp_path / root).mkdir(parents=True, exist_ok=True)
    return tmp_path


def test_repo_naming_intent_allows_intent_revealing_names(tmp_path: Path) -> None:
    repo = _mk_repo(tmp_path)
    (repo / "packages/service/catalog_loader.py").parent.mkdir(parents=True, exist_ok=True)
    (repo / "packages/service/catalog_loader.py").write_text("x = 1\n", encoding="utf-8")
    (repo / "crates/atlas/src/runtime.rs").parent.mkdir(parents=True, exist_ok=True)
    (repo / "crates/atlas/src/runtime.rs").write_text("fn main() {}\n", encoding="utf-8")
    (repo / "ops/scripts/check_health.sh").parent.mkdir(parents=True, exist_ok=True)
    (repo / "ops/scripts/check_health.sh").write_text("#!/usr/bin/env bash\n", encoding="utf-8")

    code, errors = check_naming_intent_lint(repo)
    assert code == 0
    assert errors == []


def test_repo_naming_intent_forbids_temporal_task_filenames(tmp_path: Path) -> None:
    repo = _mk_repo(tmp_path)
    offenders = [
        repo / "packages/app/phase1_runner.py",
        repo / "crates/core/src/part_2.rs",
        repo / "ops/tools/task-final.sh",
    ]
    for path in offenders:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("x\n", encoding="utf-8")

    code, errors = check_naming_intent_lint(repo)
    assert code == 1
    assert any("phase1_runner.py" in e for e in errors)
    assert any("part_2.rs" in e for e in errors)
    assert any("task-final.sh" in e for e in errors)
