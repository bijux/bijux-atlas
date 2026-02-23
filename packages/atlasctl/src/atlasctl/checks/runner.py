from __future__ import annotations

from pathlib import Path

from ..engine.runner import RunnerOptions, run_checks_payload
from .registry import list_checks


def run_checks(repo_root: Path, *, run_id: str, fail_fast: bool = False, timeout_ms: int | None = None) -> tuple[int, dict[str, object]]:
    checks = list_checks(repo_root)
    return run_checks_payload(
        repo_root,
        check_defs=checks,
        run_id=run_id,
        options=RunnerOptions(fail_fast=fail_fast, timeout_ms=timeout_ms),
    )


__all__ = ["run_checks"]
