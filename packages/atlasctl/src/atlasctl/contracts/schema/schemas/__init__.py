"""Atlasctl contract schemas and catalog access."""

from __future__ import annotations

from importlib import resources
from pathlib import Path


def schemas_root() -> Path:
    """Return the packaged schema directory path."""
    return Path(str(resources.files(__package__)))
