from __future__ import annotations

from pathlib import Path

import pytest

from bijux_atlas_scripts.core.context import RunContext
from bijux_atlas_scripts.errors import ScriptError
from bijux_atlas_scripts.fs import ensure_write_path
from bijux_atlas_scripts.reporting import write_json_report


def _ctx() -> RunContext:
    return RunContext.from_args(
        run_id="test-forbidden-writes",
        evidence_root="artifacts/evidence",
        profile="local",
        no_network=True,
    )


def test_reporting_writer_stays_in_evidence() -> None:
    ctx = _ctx()
    out = write_json_report(ctx, "make/test-forbidden-writes/unit.json", {"schema_version": 1, "ok": True})
    assert "artifacts/evidence" in out.as_posix()


def test_forbidden_ops_write_is_rejected() -> None:
    ctx = _ctx()
    with pytest.raises(ScriptError):
        ensure_write_path(ctx, Path("ops/_generated/forbidden-write.json"))
