from __future__ import annotations

from pathlib import Path

import pytest
from atlasctl.core.context import RunContext
from atlasctl.errors import ScriptError
from atlasctl.fs import ensure_write_path
from atlasctl.policy import is_forbidden_repo_path


def _ctx() -> RunContext:
    return RunContext.from_args(
        run_id="test-write-roots",
        evidence_root="artifacts/evidence",
        profile="local",
        no_network=True,
    )


def test_write_path_allowed_under_evidence_root() -> None:
    ctx = _ctx()
    target = ensure_write_path(ctx, Path("artifacts/evidence/test-write-roots/report.json"))
    assert "artifacts/evidence" in target.as_posix()


def test_write_path_rejects_ops_tree() -> None:
    ctx = _ctx()
    with pytest.raises(ScriptError):
        ensure_write_path(ctx, Path("ops/_generated/should-not-write.json"))


def test_policy_marks_source_paths_forbidden() -> None:
    ctx = _ctx()
    assert is_forbidden_repo_path(ctx, Path("docs/index.md"))
    assert is_forbidden_repo_path(ctx, Path("configs/repo/root-files-allowlist.txt"))
    assert not is_forbidden_repo_path(ctx, Path("artifacts/evidence/local/out.json"))
