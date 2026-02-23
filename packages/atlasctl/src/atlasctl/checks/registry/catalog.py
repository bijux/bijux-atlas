from __future__ import annotations

from collections import defaultdict
from pathlib import Path

from ..core.base import CheckCategory, CheckDef as CheckFactory, Severity
from .ssot import RegistryEntry, canonical_check_id, check_id_renames, legacy_checks

_CHECKS_CACHE: tuple[CheckFactory, ...] | None = None
_ALIASES_CACHE: dict[str, str] | None = None
_MARKER_VOCAB: tuple[str, ...] = ("slow", "network", "kube", "docker", "fs-write", "git")
_LEGACY_CHECKS: tuple[CheckFactory, ...] = tuple(sorted(legacy_checks(), key=lambda item: str(item.check_id)))
_LEGACY_BY_ID: dict[str, CheckFactory] = {canonical_check_id(check): check for check in _LEGACY_CHECKS}
_CALLABLE_INDEX: dict[tuple[str, str], object] = {(check.fn.__module__, check.fn.__name__): check.fn for check in _LEGACY_CHECKS}
_MODULE_CHECK_ID_INDEX: dict[tuple[str, str], object] = {(check.fn.__module__, canonical_check_id(check)): check.fn for check in _LEGACY_CHECKS}


def _build_check(**kwargs: object) -> CheckFactory:
    return CheckFactory(**kwargs)


def _from_entry(entry: RegistryEntry) -> CheckFactory:
    fn = None
    if entry.callable == "CHECKS":
        fn = _MODULE_CHECK_ID_INDEX.get((entry.module, entry.id))
    if fn is None:
        fn = _CALLABLE_INDEX.get((entry.module, entry.callable))
    if fn is None:
        legacy = _LEGACY_BY_ID.get(entry.id)
        fn = legacy.fn if legacy is not None else None
    if fn is None:
        raise ValueError(f"missing callable for check `{entry.id}`: {entry.module}:{entry.callable}")
    return _build_check(
        check_id=entry.id,
        canonical_id=entry.id,
        legacy_check_id=entry.legacy_id,
        domain=entry.domain,
        description=entry.description,
        budget_ms=entry.timeout_ms,
        fn=fn,
        severity=Severity(entry.severity),
        category=CheckCategory(entry.category),
        fix_hint=entry.fix_hint,
        intent=entry.intent,
        remediation_short=entry.remediation_short,
        remediation_link=entry.remediation_link,
        slow=(entry.speed in {"slow", "nightly"}),
        tags=tuple((*entry.groups, f"gate:{entry.gate}")),
        effects=tuple(entry.effects),
        owners=(entry.owner,),
        external_tools=tuple(entry.external_tools),
        evidence=tuple(entry.evidence),
        writes_allowed_roots=tuple(entry.writes_allowed_roots),
        result_code=entry.result_code,
    )


def _load() -> tuple[tuple[CheckFactory, ...], dict[str, str]]:
    checks = tuple(sorted(_LEGACY_CHECKS, key=lambda c: str(c.check_id)))
    aliases = {
        str(check.legacy_check_id): canonical_check_id(check)
        for check in checks
        if str(getattr(check, "legacy_check_id", "")).strip()
    }
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
    check_id = check.check_id.lower()
    if "network" in check_id:
        tags.add("network")
    if check.domain == "docker" or "docker" in check_id:
        tags.add("docker")
    if check.domain == "ops" or "kube" in check_id or "k8s" in check_id:
        tags.add("kube")
    if "write" in set(check.effects):
        tags.add("fs-write")
    if "git" in check_id:
        tags.add("git")
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


def marker_vocabulary() -> tuple[str, ...]:
    return _MARKER_VOCAB
