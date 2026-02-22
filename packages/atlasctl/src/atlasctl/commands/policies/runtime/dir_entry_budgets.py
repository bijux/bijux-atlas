"""Compatibility shim for `atlasctl.commands.policies.runtime.dir_entry_budgets`."""

from .budgets.dir_entry_budgets import (
    check_dir_entry_budgets,
    check_py_files_per_dir_budget,
    evaluate_budget,
    render_budget_text,
    report_budgets,
)

__all__ = [
    "check_dir_entry_budgets",
    "check_py_files_per_dir_budget",
    "evaluate_budget",
    "render_budget_text",
    "report_budgets",
]
