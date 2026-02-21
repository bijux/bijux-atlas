"""Checks registry accessors."""

from .catalog import check_tags, checks_by_domain, get_check, list_checks, list_domains, run_checks_for_domain

__all__ = [
    "check_tags",
    "checks_by_domain",
    "get_check",
    "list_checks",
    "list_domains",
    "run_checks_for_domain",
]
