from __future__ import annotations

from typing import Protocol

from atlasctl.core.context import RunContext

from .._contracts import OpsRuntimeRequest, OpsRuntimeResponse


class OpsRuntimeActionRunner(Protocol):
    def __call__(self, ctx: RunContext, request: OpsRuntimeRequest) -> OpsRuntimeResponse: ...
