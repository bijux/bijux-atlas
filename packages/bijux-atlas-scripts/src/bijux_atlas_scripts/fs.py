from __future__ import annotations

from pathlib import Path

from .core.context import RunContext
from .core.fs import ensure_evidence_path


def ensure_write_path(ctx: RunContext, path: Path) -> Path:
    return ensure_evidence_path(ctx, path)
