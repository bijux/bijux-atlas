from __future__ import annotations

from ..core.context import RunContext
from ..errors import ScriptError
from ..exit_codes import ERR_CONFIG


def ensure_network_allowed(ctx: RunContext, reason: str) -> None:
    if ctx.network_mode == "forbid":
        raise ScriptError(f"network access forbidden for command: {reason}", ERR_CONFIG)
