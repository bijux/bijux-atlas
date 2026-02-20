from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class OwnershipModel:
    paths: dict[str, str]
    commands: dict[str, str]

    @classmethod
    def from_json(cls, payload: dict[str, Any]) -> 'OwnershipModel':
        return cls(
            paths={str(k): str(v) for k, v in dict(payload.get('paths', {})).items()},
            commands={str(k): str(v) for k, v in dict(payload.get('commands', {})).items()},
        )


@dataclass(frozen=True)
class SurfaceModel:
    schema_version: int
    commands: list[dict[str, Any]]

    @classmethod
    def from_json(cls, payload: dict[str, Any]) -> 'SurfaceModel':
        return cls(
            schema_version=int(payload.get('schema_version', 1)),
            commands=list(payload.get('commands', [])),
        )


@dataclass(frozen=True)
class ContractsIndexModel:
    contracts: list[str]

    @classmethod
    def from_json(cls, payload: dict[str, Any]) -> 'ContractsIndexModel':
        return cls(contracts=[str(x) for x in payload.get('contracts', [])])
