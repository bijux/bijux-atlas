"""Atlasctl contracts package."""
from __future__ import annotations

from ..core.context import RunContext
from ..cli.registry import domain_payload


def run(ctx: RunContext) -> dict[str, object]:
    return domain_payload(ctx, "contracts")
