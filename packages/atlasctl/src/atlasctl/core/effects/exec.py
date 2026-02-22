"""Compatibility shim for `atlasctl.core.effects.exec`."""

from __future__ import annotations

from ..exec import check_output, run

__all__ = ["check_output", "run"]
