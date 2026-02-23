"""Atlasctl registry package."""
from __future__ import annotations

from .checks import CheckRecord
from .commands import CommandRecord
from .suites import SuiteRecord
from .loader import load as load_registry
from .models import Registry, RegistryCheck, RegistryCommand, RegistrySuite
from ..core.context import RunContext
from ..cli.surface_registry import domain_payload


def run(ctx: RunContext) -> dict[str, object]:
    return domain_payload(ctx, "registry")


__all__ = [
    "CheckRecord",
    "CommandRecord",
    "SuiteRecord",
    "Registry",
    "RegistryCheck",
    "RegistryCommand",
    "RegistrySuite",
    "load_registry",
    "run",
]
