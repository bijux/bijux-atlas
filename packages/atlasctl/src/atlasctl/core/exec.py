"""Compatibility shim for `atlasctl.core.exec`."""

from .effects.exec import check_output, run

__all__ = ["check_output", "run"]
