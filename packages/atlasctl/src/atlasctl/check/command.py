"""Compatibility shim for check command module."""

from ..commands.check import legacy as _legacy

globals().update(vars(_legacy))
