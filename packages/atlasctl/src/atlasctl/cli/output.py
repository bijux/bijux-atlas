"""CLI payload output helpers."""

from __future__ import annotations

from ..core.serialize import dumps_json


def emit(payload: dict[str, object], as_json: bool) -> None:
    print(dumps_json(payload, pretty=not as_json))


def build_base_payload(ctx, status: str = "ok") -> dict[str, object]:
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": status,
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        "repo_root": str(ctx.repo_root),
        "run_dir": str(ctx.run_dir),
        "evidence_root": str(ctx.evidence_root),
        "scripts_artifact_root": str(ctx.scripts_artifact_root),
        "network": ctx.network_mode,
        "format": ctx.output_format,
        "git_sha": ctx.git_sha,
        "git_dirty": ctx.git_dirty,
    }
