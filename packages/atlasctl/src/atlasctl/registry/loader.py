"""Single registry loader surface."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .spine import (
    REGISTRY_SPINE_GENERATED_JSON,
    generate_registry_spine,
    load_registry,
)


def load(repo_root: Path | None = None):
    return load_registry(repo_root)


def generate(repo_root: Path | None = None) -> dict[str, Any]:
    return generate_registry_spine(repo_root)


__all__ = ["REGISTRY_SPINE_GENERATED_JSON", "generate", "load"]
