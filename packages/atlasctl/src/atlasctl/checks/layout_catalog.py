"""Compatibility shim for `atlasctl.checks.layout_catalog`."""

from .registry.layout_catalog import LayoutCheckSpec, list_layout_checks

__all__ = ["LayoutCheckSpec", "list_layout_checks"]
