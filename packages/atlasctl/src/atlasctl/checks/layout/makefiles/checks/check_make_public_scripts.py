"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_make_public_scripts`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_make_public_scripts")
ROOT = _IMPL.ROOT
patterns = _IMPL.patterns
violations = _IMPL.violations

__all__ = ["ROOT", "patterns", "violations"]
