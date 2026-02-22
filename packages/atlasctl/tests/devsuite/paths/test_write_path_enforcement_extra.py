from __future__ import annotations

from pathlib import Path

import pytest
from atlasctl.core.context import RunContext
from atlasctl.core.errors import ScriptError
from atlasctl.core.fs import ensure_evidence_path


def _ctx() -> RunContext:
    return RunContext.from_args("test-write-paths", "artifacts/evidence", "test", True)


def test_forbidden_docs_write_is_rejected() -> None:
    ctx = _ctx()
    with pytest.raises(ScriptError):
        ensure_evidence_path(ctx, Path("docs/_generated/forbidden.json"))


def test_forbidden_configs_write_is_rejected() -> None:
    ctx = _ctx()
    with pytest.raises(ScriptError):
        ensure_evidence_path(ctx, Path("configs/forbidden.json"))
