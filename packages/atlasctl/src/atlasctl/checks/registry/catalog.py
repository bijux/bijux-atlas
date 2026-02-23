from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

from ..model import CheckCategory, CheckDef as CheckFactory, Severity
from .ssot import RegistryEntry, canonical_check_id, check_id_renames, legacy_checks

_MARKER_VOCAB: tuple[str, ...] = ("slow", "network", "kube", "docker", "fs-write", "git")


@dataclass(frozen=True)
class RegistryIndex:
    checks: tuple[CheckFactory, ...]
    aliases: dict[str, str]
    by_id: dict[str, CheckFactory]
    by_domain: dict[str, tuple[CheckFactory, ...]]


def _build_check(**kwargs: object) -> CheckFactory:
    return CheckFactory(**kwargs)


def _from_entry(entry: RegistryEntry, module_callable_index: dict[tuple[str, str], object], module_check_id_index: dict[tuple[str, str], object], legacy_by_id: dict[str, CheckFactory]) -> CheckFactory:
    fn = None
    if entry.callable == "CHECKS":
        fn = module_check_id_index.get((entry.module, entry.id))
    if fn is None:
        fn = module_callable_index.get((entry.module, entry.callable))
    if fn is None:
        legacy = legacy_by_id.get(entry.id)
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


@lru_cache(maxsize=1)
def _build_index() -> RegistryIndex:
    legacy = tuple(sorted(legacy_checks(), key=lambda item: str(item.check_id)))
    legacy_by_id = {canonical_check_id(check): check for check in legacy}
    module_callable_index = {(check.fn.__module__, check.fn.__name__): check.fn for check in legacy}
    module_check_id_index = {(check.fn.__module__, canonical_check_id(check)): check.fn for check in legacy}
    checks = tuple(sorted(legacy, key=lambda c: str(c.check_id)))
    aliases = {
        str(check.legacy_check_id): canonical_check_id(check)
        for check in checks
        if str(getattr(check, "legacy_check_id", "")).strip()
    }
    aliases.update(check_id_renames())
    by_id = {str(check.check_id): check for check in checks}
    by_domain: dict[str, tuple[CheckFactory, ...]] = {}
    grouped: dict[str, list[CheckFactory]] = defaultdict(list)
    for check in checks:
        grouped[str(check.domain)].append(check)
    for domain, rows in grouped.items():
        by_domain[domain] = tuple(sorted(rows, key=lambda row: str(row.check_id)))
    # Touch helper for static checks and future generator use.
    _ = (_from_entry, module_callable_index, module_check_id_index, legacy_by_id)
    return RegistryIndex(checks=checks, aliases=aliases, by_id=by_id, by_domain=by_domain)


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
    return _build_index().checks


def list_domains() -> list[str]:
    return sorted({"all", *{c.domain for c in list_checks()}})


def checks_by_domain() -> dict[str, list[CheckFactory]]:
    return {domain: list(rows) for domain, rows in _build_index().by_domain.items()}


def run_checks_for_domain(repo_root: Path, domain: str) -> list[CheckFactory]:
    if domain == "all":
        return list(list_checks())
    return [c for c in list_checks() if c.domain == domain]


def get_check(check_id: str) -> CheckFactory | None:
    index = _build_index()
    resolved = index.aliases.get(check_id, check_id)
    return index.by_id.get(resolved)


def check_rename_aliases() -> dict[str, str]:
    return dict(sorted(_build_index().aliases.items()))


def marker_vocabulary() -> tuple[str, ...]:
    return _MARKER_VOCAB
