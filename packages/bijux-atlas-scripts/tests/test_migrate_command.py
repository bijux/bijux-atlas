from __future__ import annotations

import argparse
from pathlib import Path

from bijux_atlas_scripts.core.context import RunContext
from bijux_atlas_scripts.migrate.command import run_migrate_command


def _ctx(root: Path) -> RunContext:
    return RunContext(
        run_id="r1",
        profile="local",
        repo_root=root,
        evidence_root=root / "artifacts/evidence",
        scripts_artifact_root=root / "artifacts/bijux-atlas-scripts/run/r1",
        run_dir=root / "artifacts/evidence/r1",
        output_format="text",
        network_mode="allow",
        verbose=False,
        quiet=False,
        require_clean_git=False,
        no_network=False,
        git_sha="unknown",
        git_dirty=False,
    )


def test_migrate_layout_runs_sequence(monkeypatch, tmp_path: Path) -> None:
    calls: list[list[str]] = []

    class Dummy:
        returncode = 0

    def fake_run(cmd, cwd=None, text=None, check=None):  # type: ignore[no-untyped-def]
        calls.append(list(cmd))
        return Dummy()

    monkeypatch.setattr("bijux_atlas_scripts.migrate.command.subprocess.run", fake_run)
    ns = argparse.Namespace(migrate_cmd="layout")
    assert run_migrate_command(_ctx(tmp_path), ns) == 0
    assert calls and calls[0][:2] == ["bash", "scripts/areas/internal/migrate_paths.sh"]

