"""Compatibility shim for docs command module."""

from ..commands.docs import legacy as _legacy

globals().update(vars(_legacy))
