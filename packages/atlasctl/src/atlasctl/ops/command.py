"""Compatibility shim for ops command module."""

from ..commands.ops import legacy as _legacy

globals().update(vars(_legacy))
