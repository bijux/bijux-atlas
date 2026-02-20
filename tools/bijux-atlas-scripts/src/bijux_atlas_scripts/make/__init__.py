from __future__ import annotations

from ..core.context import RunContext
from ..domain_cmd import domain_payload


def run(ctx: RunContext) -> dict[str, object]:
    return domain_payload(ctx, "make")
