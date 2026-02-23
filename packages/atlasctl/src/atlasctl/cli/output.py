"""CLI payload output helpers."""

from __future__ import annotations

from datetime import date

from ..core.runtime.serialize import dumps_json
from .render import render_error


def emit(payload: dict[str, object], as_json: bool) -> None:
    print(dumps_json(payload, pretty=not as_json))


def build_base_payload(ctx, status: str = "ok") -> dict[str, object]:
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": status,
        "ok": status == "ok",
        "errors": [],
        "warnings": [],
        "meta": {},
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


def resolve_output_format(*, cli_json: bool, cli_format: str | None, ci_present: bool) -> str:
    if cli_json:
        return "json"
    if cli_format:
        return cli_format
    return "json" if ci_present else "text"


def no_network_flag_expired(today: date, expiry: date) -> bool:
    return today > expiry
