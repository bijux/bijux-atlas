"""Compatibility shim for `atlasctl.core.network`."""

from .effects.network import check_network_allowed

__all__ = ["check_network_allowed"]
