from __future__ import annotations

from dataclasses import dataclass
from fnmatch import fnmatch
from pathlib import Path
from subprocess import run

from .model import CheckDef, CheckSelector
from .registry import check_tags


@dataclass(frozen=True)
class SelectionCriteria:
    domain: str = ""
    id_globs: tuple[str, ...] = ()
    tags: tuple[str, ...] = ()
    exclude_tags: tuple[str, ...] = ()
    owners: tuple[str, ...] = ()
    include_internal: bool = False
    only_slow: bool = False
    only_fast: bool = False
    changed_only: bool = False
    changed_paths: tuple[str, ...] = ()
    query: str = ""


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


def _changed_paths(repo_root: Path) -> tuple[str, ...]:
    proc = run(["git", "diff", "--name-only", "HEAD"], cwd=repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        return ()
    return tuple(sorted(line.strip() for line in proc.stdout.splitlines() if line.strip()))


def parse_selection_criteria(ns: object, repo_root: Path) -> SelectionCriteria:
    def _coerce(value: object) -> str:
        if value is None:
            return ""
        return str(value).strip()

    raw_tags = tuple(str(item).strip() for item in (getattr(ns, "marker", []) or []) if str(item).strip())
    raw_exclude_tags = tuple(
        str(item).strip() for item in ((getattr(ns, "exclude_marker", []) or []) + (getattr(ns, "exclude_tag", []) or [])) if str(item).strip()
    )
    criteria = SelectionCriteria(
        domain=_coerce(getattr(ns, "domain_filter", "")),
        id_globs=tuple(
            _coerce(item)
            for item in (
                [getattr(ns, "id", "")]
                + [getattr(ns, "select", "")]
                + [getattr(ns, "check_target", "")]
            )
            if _coerce(item)
        ),
        tags=raw_tags + tuple(str(item).strip() for item in (getattr(ns, "tag", []) or []) if str(item).strip()),
        exclude_tags=raw_exclude_tags,
        owners=tuple(str(item).strip() for item in (getattr(ns, "owner", []) or []) if str(item).strip()),
        include_internal=bool(getattr(ns, "include_internal", False)),
        only_slow=bool(getattr(ns, "only_slow", False)),
        only_fast=bool(getattr(ns, "only_fast", False)),
        changed_only=bool(getattr(ns, "changed_only", False)),
        query=_coerce(getattr(ns, "k", "")),
    )
    if criteria.changed_only:
        return SelectionCriteria(**{**criteria.__dict__, "changed_paths": _changed_paths(repo_root)})
    return criteria


def _is_internal(check: CheckDef) -> bool:
    tags = set(check_tags(check))
    return "internal" in tags or "internal-only" in tags


def apply_selection_criteria(checks: list[CheckDef], criteria: SelectionCriteria) -> list[CheckDef]:
    out: list[CheckDef] = []
    for check in checks:
        if criteria.domain and str(check.domain) != criteria.domain:
            continue
        legacy_id = str(check.check_id)
        canonical_id = legacy_id if legacy_id.startswith("checks_") else f"checks_{legacy_id.replace('.', '_')}"
        if criteria.id_globs and not any(
            fnmatch(str(check.id), glob)
            or str(check.id) == glob
            or fnmatch(legacy_id, glob)
            or legacy_id == glob
            or fnmatch(canonical_id, glob)
            or canonical_id == glob
            for glob in criteria.id_globs
        ):
            continue
        if criteria.query and criteria.query not in str(check.id) and criteria.query not in str(check.title):
            continue
        tags = set(check_tags(check))
        if criteria.tags and not set(criteria.tags).issubset(tags):
            continue
        if criteria.exclude_tags and tags.intersection(criteria.exclude_tags):
            continue
        if criteria.owners and not set(criteria.owners).intersection(set(str(owner) for owner in check.owners)):
            continue
        if criteria.only_slow and not bool(check.slow):
            continue
        if criteria.only_fast and bool(check.slow):
            continue
        if not criteria.include_internal and _is_internal(check):
            continue
        if criteria.changed_only and criteria.changed_paths:
            module_path = str(check.fn.__module__).replace(".", "/")
            if not any(path.endswith(".py") and module_path in path for path in criteria.changed_paths):
                continue
        out.append(check)
    return sorted(out, key=lambda check: str(check.id))


__all__ = [
    "SelectionCriteria",
    "apply_selection_criteria",
    "match_selector",
    "parse_selection_criteria",
    "select_checks",
]
