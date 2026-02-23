from __future__ import annotations

from fnmatch import fnmatch

from .model import CheckDef, CheckSelector


def match_selector(check: CheckDef, selector: CheckSelector) -> bool:
    if selector.ids and str(check.id) not in {str(item) for item in selector.ids}:
        return False
    if selector.domains and str(check.domain) not in {str(item) for item in selector.domains}:
        return False
    if selector.tags and not ({str(item) for item in selector.tags} & set(check.tags)):
        return False
    if selector.patterns and not any(fnmatch(str(check.id), pattern) for pattern in selector.patterns):
        return False
    return True


def select_checks(checks: list[CheckDef], selector: CheckSelector) -> list[CheckDef]:
    matched = [check for check in checks if match_selector(check, selector)]
    return sorted(matched, key=lambda check: check.canonical_key)


__all__ = ["match_selector", "select_checks"]
