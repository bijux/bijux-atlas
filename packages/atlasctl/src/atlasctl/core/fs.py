from __future__ import annotations

from pathlib import Path

from ..errors import ScriptError
from ..exit_codes import ERR_ARTIFACT
from .context import RunContext


def ensure_evidence_path(ctx: RunContext, path: Path) -> Path:
    resolved = path.resolve() if path.is_absolute() else (ctx.repo_root / path).resolve()
    forbidden = (ctx.repo_root / "ops").resolve()
    if resolved == forbidden or forbidden in resolved.parents:
        raise ScriptError(f"forbidden write path under ops/: {resolved}", ERR_ARTIFACT)
    allowed = ctx.evidence_root.resolve()
    if resolved == allowed or allowed in resolved.parents:
        resolved.parent.mkdir(parents=True, exist_ok=True)
        return resolved
    raise ScriptError(f"forbidden write path outside evidence root: {resolved}", ERR_ARTIFACT)
