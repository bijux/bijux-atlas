"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.ops.ops_checks.impl.check_ops_single_owner_contracts`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.ops.ops_checks.impl.check_ops_single_owner_contracts")
main = _IMPL.main

__all__ = ["main"]
