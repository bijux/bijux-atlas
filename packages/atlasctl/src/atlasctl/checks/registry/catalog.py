from __future__ import annotations

import importlib
from collections import defaultdict
from pathlib import Path

from ..core.base import CheckCategory, CheckDef as CheckFactory, Severity
from .ssot import RegistryEntry, check_id_renames, legacy_check_by_id, load_registry_entries

_CHECKS_CACHE: tuple[CheckFactory, ...] | None = None
_ALIASES_CACHE: dict[str, str] | None = None


def _build_check(**kwargs: object) -> CheckFactory:
    return CheckFactory(**kwargs)


def _from_entry(entry: RegistryEntry) -> CheckFactory:
    module = importlib.import_module(entry.module)
    fn = getattr(module, entry.callable, None)
    if fn is None:
        legacy = legacy_check_by_id().get(entry.id)
        fn = legacy.fn if legacy is not None else None
    if fn is None:
        raise ValueError(f"missing callable for check `{entry.id}`: {entry.module}:{entry.callable}")
    return _build_check(
        check_id=entry.id,
        legacy_check_id=entry.legacy_id,
        domain=entry.domain,
        description=entry.description,
        budget_ms=entry.timeout_ms,
        fn=fn,
        severity=Severity(entry.severity),
        category=CheckCategory(entry.category),
        fix_hint=entry.fix_hint,
        slow=(entry.speed == "slow"),
        tags=tuple((*entry.groups, f"gate:{entry.gate}")),
        effects=tuple(entry.effects),
        owners=(entry.owner,),
        external_tools=tuple(entry.external_tools),
        evidence=tuple(entry.evidence),
        writes_allowed_roots=tuple(entry.writes_allowed_roots),
    )


def _load() -> tuple[tuple[CheckFactory, ...], dict[str, str]]:
    entries = load_registry_entries()
    checks = tuple(sorted((_from_entry(entry) for entry in entries), key=lambda c: c.check_id))
    aliases = {entry.legacy_id: entry.id for entry in entries if entry.legacy_id}
    aliases.update(check_id_renames())
    return checks, aliases


def _ensure() -> None:
    global _CHECKS_CACHE, _ALIASES_CACHE
    if _CHECKS_CACHE is None or _ALIASES_CACHE is None:
        checks, aliases = _load()
        _CHECKS_CACHE = checks
        _ALIASES_CACHE = aliases


def check_tags(check: CheckFactory) -> tuple[str, ...]:
    tags = set(check.tags)
    tags.add(check.domain)
    tags.add("slow" if check.slow else "fast")
    if "internal" not in tags and "internal-only" not in tags:
        tags.add("required")
    return tuple(sorted(tags))


def list_checks() -> tuple[CheckFactory, ...]:
    _ensure()
    assert _CHECKS_CACHE is not None
    return _CHECKS_CACHE


def list_domains() -> list[str]:
    return sorted({"all", *{c.domain for c in list_checks()}})


def checks_by_domain() -> dict[str, list[CheckFactory]]:
    grouped: dict[str, list[CheckFactory]] = defaultdict(list)
    for check in list_checks():
        grouped[check.domain].append(check)
    return dict(grouped)


def run_checks_for_domain(repo_root: Path, domain: str) -> list[CheckFactory]:
    if domain == "all":
        return list(list_checks())
    return [c for c in list_checks() if c.domain == domain]


def get_check(check_id: str) -> CheckFactory | None:
    _ensure()
    assert _ALIASES_CACHE is not None
    if check_id in _ALIASES_CACHE:
        check_id = _ALIASES_CACHE[check_id]
    for check in list_checks():
        if check.check_id == check_id:
            return check
    return None


def check_rename_aliases() -> dict[str, str]:
    _ensure()
    assert _ALIASES_CACHE is not None
    return dict(sorted(_ALIASES_CACHE.items()))
