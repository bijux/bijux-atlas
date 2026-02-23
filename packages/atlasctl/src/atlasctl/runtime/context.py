"""Canonical runtime context module.

Use `atlasctl.runtime.context.RunContext` as the primary import.
`atlasctl.core.context` remains a temporary LEGACY facade during the migration
window (cutoff: 2026-04-01).
"""
from __future__ import annotations

from atlasctl.core.context import RunContext

__all__ = ["RunContext"]
