from __future__ import annotations

from datetime import datetime, timezone


def build_run_id(git_sha: str, prefix: str = "atlas") -> str:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")
    return f"{prefix}-{stamp}-{git_sha}"
