"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_make_targets_catalog_drift`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_make_targets_catalog_drift")
main = _IMPL.main

__all__ = ["main"]
