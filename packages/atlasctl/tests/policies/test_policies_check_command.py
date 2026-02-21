from __future__ import annotations

import argparse
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.policies import command as policies_command


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


def test_policies_check_does_not_invoke_recursive_make_policy_targets(tmp_path: Path, monkeypatch) -> None:
    ns = argparse.Namespace(
        policies_cmd="check",
        report="json",
        emit_artifacts=False,
        fail_fast=False,
    )
    commands: list[list[str]] = []

    def _fake_run(cmd: list[str], repo_root: Path) -> tuple[int, str]:
        assert repo_root == tmp_path
        commands.append(cmd)
        return 0, ""

    monkeypatch.setattr(policies_command, "_run", _fake_run)
    assert policies_command.run_policies_command(_ctx(tmp_path), ns) == 0
    assert all(not (cmd[:2] == ["make", "-s"]) for cmd in commands)

