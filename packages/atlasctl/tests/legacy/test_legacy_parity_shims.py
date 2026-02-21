from __future__ import annotations

from atlasctl.core.fs import ensure_evidence_path
from atlasctl.legacy import evidence_policy as legacy_evidence_policy
from atlasctl.legacy import runner as legacy_runner
from atlasctl.legacy.effects import filesystem as legacy_effects_filesystem
from atlasctl.legacy.effects import process as legacy_effects_process


def test_legacy_shim_exports_align_with_replacements() -> None:
    assert legacy_evidence_policy.ensure_evidence_path is ensure_evidence_path
    assert callable(legacy_runner.run_legacy_script)
    assert callable(legacy_effects_filesystem.ensure_write_path)
    assert callable(legacy_effects_process.run)
