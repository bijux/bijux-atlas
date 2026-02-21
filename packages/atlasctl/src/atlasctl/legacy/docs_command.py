"""Legacy docs command compatibility shim."""

from ..commands.docs import legacy as _legacy

globals().update(vars(_legacy))
