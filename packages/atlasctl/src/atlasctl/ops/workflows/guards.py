from __future__ import annotations

import os

from atlasctl.core.context import RunContext

from atlasctl.ops.adapters import kubectl


def ensure_local_kind_context(ctx: RunContext) -> tuple[bool, str]:
    result = kubectl.run(ctx, "config", "current-context")
    context = result.combined_output.strip()
    if result.code != 0:
        return False, f"failed to resolve kubectl context: {context or 'unknown'}"
    if not context.startswith("kind-"):
        return False, f"local ops command requires kind context; current={context or 'unknown'}"
    return True, context


def require_destructive_ops_allowed(action: str) -> tuple[bool, str]:
    raw = str(os.environ.get("ATLASCTL_OPS_ALLOW_DESTRUCTIVE", "")).strip().lower()
    if raw in {"1", "true", "yes", "on"}:
        return True, ""
    return False, (
        f"refusing destructive ops action `{action}`; set ATLASCTL_OPS_ALLOW_DESTRUCTIVE=1 to proceed"
    )
