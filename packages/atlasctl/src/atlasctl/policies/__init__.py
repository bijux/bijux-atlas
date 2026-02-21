"""Atlasctl policies package."""
from __future__ import annotations

from ..core.context import RunContext
from ..cli.surface_registry import domain_payload


def run(ctx: RunContext) -> dict[str, object]:
    return domain_payload(ctx, "policies")
