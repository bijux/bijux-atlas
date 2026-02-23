"""Registry selector helpers."""

from __future__ import annotations

from .loader import load
from .models import RegistryCheck, RegistryCommand


def select_checks(*, domain: str | None = None, tags: tuple[str, ...] = (), severity: str | None = None, suite: str | None = None) -> tuple[RegistryCheck, ...]:
    return load().select_checks(domain=domain, tags=tags, severity=severity, suite=suite)


def select_commands(*, group: str | None = None, tags: tuple[str, ...] = ()) -> tuple[RegistryCommand, ...]:
    return load().select_commands(group=group, tags=tags)


__all__ = ["select_checks", "select_commands"]
