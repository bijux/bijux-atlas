from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class PortMapping:
    name: str
    port: int
    protocol: str = "tcp"


@dataclass(frozen=True)
class NamespaceConfig:
    name: str
    purpose: str
