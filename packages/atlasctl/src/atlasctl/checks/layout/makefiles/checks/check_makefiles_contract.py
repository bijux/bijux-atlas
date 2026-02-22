"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_makefiles_contract`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_makefiles_contract")
check_makefiles_contract = _IMPL.check_makefiles_contract

__all__ = ["check_makefiles_contract"]
