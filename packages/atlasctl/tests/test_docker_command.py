from __future__ import annotations

import argparse
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.docker.command import run_docker_command


def test_docker_scan_invokes_canonical_script(monkeypatch, tmp_path: Path) -> None:
    calls: list[list[str]] = []

    class Dummy:
        returncode = 0

    def fake_run(cmd, cwd=None, text=None, check=None):  # type: ignore[no-untyped-def]
        calls.append(list(cmd))
        assert cwd is not None
        return Dummy()

    monkeypatch.setattr("atlasctl.docker.command.subprocess.run", fake_run)
    ctx = RunContext(
        run_id="r1",
        profile="local",
        repo_root=tmp_path,
        evidence_root=tmp_path / "artifacts/evidence",
        scripts_artifact_root=tmp_path / "artifacts/atlasctl/run/r1",
        run_dir=tmp_path / "artifacts/evidence/r1",
        output_format="text",
        network_mode="allow",
        verbose=False,
        quiet=False,
        require_clean_git=False,
        no_network=False,
        git_sha="unknown",
        git_dirty=False,
    )
    ns = argparse.Namespace(docker_cmd="scan", image="x:y")
    assert run_docker_command(ctx, ns) == 0
    assert calls and calls[0][0] == "docker/scripts/docker-scan.sh"
