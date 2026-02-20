from __future__ import annotations

from ..domain_cmd import domain_payload
from ..run_context import RunContext


def run(ctx: RunContext) -> dict[str, object]:
    return domain_payload(ctx, "docs")
