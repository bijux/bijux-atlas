"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.ops.ops_checks.impl.check_ops_cross_area_script_refs`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.ops.ops_checks.impl.check_ops_cross_area_script_refs")
main = _IMPL.main

__all__ = ["main"]
