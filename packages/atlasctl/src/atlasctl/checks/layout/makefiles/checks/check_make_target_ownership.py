"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_make_target_ownership`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_make_target_ownership")
check_make_target_ownership_complete = _IMPL.check_make_target_ownership_complete

__all__ = ["check_make_target_ownership_complete"]
