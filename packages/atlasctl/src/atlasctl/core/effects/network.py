"""Compatibility shim for `atlasctl.core.effects.network`.

LEGACY shim (cutoff: 2026-04-01). Prefer `atlasctl.core.network`.
"""

from __future__ import annotations

from ..network import http_get

__all__ = ["http_get"]
