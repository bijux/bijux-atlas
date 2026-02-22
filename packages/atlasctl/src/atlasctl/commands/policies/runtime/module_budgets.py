"""Compatibility shim for `atlasctl.commands.policies.runtime.module_budgets`."""

from .budgets.module_budgets import check_modules_per_domain

__all__ = ["check_modules_per_domain"]
