"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_makefile_target_boundaries`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_makefile_target_boundaries")
check_make_target_boundaries_enforced = _IMPL.check_make_target_boundaries_enforced

__all__ = ["check_make_target_boundaries_enforced"]
