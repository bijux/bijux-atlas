"""Canonical result import surface.

Use `atlasctl.core.result` from command/runtime code instead of importing from
`core.runtime.result` directly. This keeps the public typing surface stable even
if the implementation module moves.
"""

from __future__ import annotations

from .runtime.result import Err, Ok, Result

__all__ = ["Ok", "Err", "Result"]
