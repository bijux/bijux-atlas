"""Compatibility shim for `atlasctl.commands.policies.runtime.budget_handlers`."""

from .budgets.handlers import handle_budget_command

__all__ = ["handle_budget_command"]
