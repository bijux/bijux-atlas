from __future__ import annotations

from typing import Callable

from atlasctl.core.context import RunContext
from atlasctl.core.runtime.guards.network_guard import install_no_network_guard


def install_runtime_capabilities(ctx: RunContext) -> Callable[[], None] | None:
    if ctx.no_network:
        from atlasctl.core.runtime.guards.env_guard import guard_no_network_mode

        guard_no_network_mode(True)
        return install_no_network_guard()
    return None

