"""Compatibility shim for `atlasctl.checks.runner`."""

from .engine.runner import domains, run_domain

__all__ = ["domains", "run_domain"]
