"""Checks registry accessors."""

from .catalog import check_rename_aliases, check_tags, checks_by_domain, get_check, list_checks, list_domains, marker_vocabulary, run_checks_for_domain

__all__ = [
    "check_tags",
    "check_rename_aliases",
    "checks_by_domain",
    "get_check",
    "list_checks",
    "list_domains",
    "marker_vocabulary",
    "run_checks_for_domain",
]
