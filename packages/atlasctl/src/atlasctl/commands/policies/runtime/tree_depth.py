"""Compatibility shim for `atlasctl.commands.policies.runtime.tree_depth`."""

from .budgets.tree_depth import check_tree_depth

__all__ = ["check_tree_depth"]
