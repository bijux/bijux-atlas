"""Compatibility shim; canonical implementation moved to `atlasctl.checks.domains.policies.make.impl.check_makefile_headers`."""

from importlib import import_module

_IMPL = import_module("atlasctl.checks.domains.policies.make.impl.check_makefile_headers")
check_makefiles_help_has_descriptions = _IMPL.check_makefiles_help_has_descriptions
check_makefiles_tab_indentation = _IMPL.check_makefiles_tab_indentation

__all__ = ["check_makefiles_help_has_descriptions", "check_makefiles_tab_indentation"]
