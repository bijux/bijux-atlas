from __future__ import annotations

import argparse
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.migrate.command import run_migrate_command


def _ctx(root: Path) -> RunContext:
    return RunContext(
        run_id="r1",
        profile="local",
        repo_root=root,
        evidence_root=root / "artifacts/evidence",
        scripts_artifact_root=root / "artifacts/atlasctl/run/r1",
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


def test_migrate_layout_rewrites_paths_and_checks_layout(tmp_path: Path, monkeypatch) -> None:  # type: ignore[no-untyped-def]
    tracked = tmp_path / "docs" / "x.md"
    tracked.parent.mkdir(parents=True, exist_ok=True)
    tracked.write_text("use ./charts/ and docs/operations/ops/foo.md\n", encoding="utf-8")

    class Dummy:
        returncode = 0
        stdout = "docs/x.md\n"

    def fake_run(cmd, cwd=None, text=None, capture_output=None, check=None):  # type: ignore[no-untyped-def]
        return Dummy()

    monkeypatch.setattr("atlasctl.migrate.command.subprocess.run", fake_run)
    monkeypatch.setattr("atlasctl.migrate.command.check_layout_contract", lambda _: (0, []))
    ns = argparse.Namespace(migrate_cmd="layout", json=False)
    assert run_migrate_command(_ctx(tmp_path), ns) == 0
    rewritten = tracked.read_text(encoding="utf-8")
    assert "./ops/k8s/charts/" in rewritten
    assert "docs/operations/foo.md" in rewritten
