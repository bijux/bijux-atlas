from __future__ import annotations

from pathlib import Path


def ops_run_root(ctx) -> Path:  # noqa: ANN001
    return (ctx.repo_root / "artifacts" / "runs" / str(ctx.run_id) / "ops").resolve()


def ops_run_area_dir(ctx, area: str) -> Path:  # noqa: ANN001
    return ops_run_root(ctx) / area
