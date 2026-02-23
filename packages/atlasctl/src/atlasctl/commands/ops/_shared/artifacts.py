from __future__ import annotations

from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.core.fs import ensure_evidence_path


def ops_evidence_dir(ctx: RunContext, area: str) -> Path:
    """Canonical ops evidence layout: artifacts/evidence/<area>/<run_id>/..."""
    return ensure_evidence_path(ctx, ctx.evidence_root / area / ctx.run_id)
