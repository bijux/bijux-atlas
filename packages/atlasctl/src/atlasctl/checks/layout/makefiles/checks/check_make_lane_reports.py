"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_make_lane_reports`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_make_lane_reports")
main = _IMPL.main

__all__ = ["main"]
