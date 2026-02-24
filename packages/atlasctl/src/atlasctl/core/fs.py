from __future__ import annotations

import json
from pathlib import Path

from .errors import ScriptError
from .exit_codes import ERR_ARTIFACT
from .context import RunContext


def ensure_evidence_path(ctx: RunContext, path: Path) -> Path:
    resolved = path.resolve() if path.is_absolute() else (ctx.repo_root / path).resolve()
    forbidden = (ctx.repo_root / "ops").resolve()
    if resolved == forbidden or forbidden in resolved.parents:
        raise ScriptError(f"forbidden write path under ops/: {resolved}", ERR_ARTIFACT, kind="forbidden_write_path")
    allowed_roots = (
        ctx.evidence_root.resolve(),
        (ctx.repo_root / "artifacts/atlasctl/checks").resolve(),
    )
    if any(resolved == root or root in resolved.parents for root in allowed_roots):
        resolved.parent.mkdir(parents=True, exist_ok=True)
        return resolved
    raise ScriptError(f"forbidden write path outside evidence root: {resolved}", ERR_ARTIFACT, kind="forbidden_write_path")


def ensure_managed_write_path(ctx: RunContext, path: Path) -> Path:
    resolved = path.resolve() if path.is_absolute() else (ctx.repo_root / path).resolve()
    forbidden = (ctx.repo_root / "ops").resolve()
    if resolved == forbidden or forbidden in resolved.parents:
        raise ScriptError(f"forbidden write path under ops/: {resolved}", ERR_ARTIFACT, kind="forbidden_write_path")
    allowed_roots = (ctx.evidence_root.resolve(), ctx.scripts_artifact_root.resolve())
    if any(resolved == root or root in resolved.parents for root in allowed_roots):
        resolved.parent.mkdir(parents=True, exist_ok=True)
        return resolved
    raise ScriptError(f"forbidden write path outside managed roots: {resolved}", ERR_ARTIFACT, kind="forbidden_write_path")


def write_text(ctx: RunContext, path: Path, content: str, encoding: str = "utf-8") -> Path:
    out = ensure_managed_write_path(ctx, path)
    out.write_text(content, encoding=encoding)
    record_artifact_write(ctx, out)
    return out


def write_json(ctx: RunContext, path: Path, payload: dict[str, object]) -> Path:
    return write_text(ctx, path, json.dumps(payload, indent=2, sort_keys=True) + "\n")


def record_artifact_write(ctx: RunContext, path: Path) -> None:
    meta = ensure_evidence_path(ctx, ctx.evidence_root / "metadata" / ctx.run_id / "artifact-writes.jsonl")
    rel = path.resolve().relative_to(ctx.repo_root)
    with meta.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps({"run_id": ctx.run_id, "path": rel.as_posix()}, sort_keys=True) + "\n")
