"""Legacy ops command compatibility shim."""

from ..commands.ops import legacy as _legacy

globals().update(vars(_legacy))
