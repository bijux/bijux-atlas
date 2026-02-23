"""Checks registry SSOT accessors."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import date

from ..core.base import CheckDef, CheckId, DomainId, Tag
from ..domains.configs import CHECKS as CONFIGS_CHECKS
from ..domains.docs import CHECKS as DOCS_CHECKS
from ..domains.internal import CHECKS as INTERNAL_CHECKS
from ..domains.ops import CHECKS as OPS_CHECKS
from ..domains.policies import CHECKS as POLICIES_CHECKS
from ..domains.repo import CHECKS as REPO_CHECKS
from .catalog import check_rename_aliases, check_tags, checks_by_domain, marker_vocabulary, run_checks_for_domain
from .ssot import check_id_alias_expiry

ALL_CHECKS: tuple[CheckDef, ...] = tuple(
    sorted(
        (
            *REPO_CHECKS,
            *OPS_CHECKS,
            *POLICIES_CHECKS,
            *DOCS_CHECKS,
            *CONFIGS_CHECKS,
            *INTERNAL_CHECKS,
        ),
        key=lambda check: str(check.check_id),
    )
)
TAGS_VOCAB: frozenset[Tag] = frozenset(Tag(tag) for check in ALL_CHECKS for tag in check_tags(check))
MARKERS_VOCAB: frozenset[Tag] = frozenset(Tag(marker) for marker in marker_vocabulary())
_CHECK_INDEX: dict[str, CheckDef] = {str(check.check_id): check for check in ALL_CHECKS}


@dataclass(frozen=True)
class CheckAlias:
    old: CheckId
    new: CheckId
    expires_on: date


def list_checks() -> tuple[CheckDef, ...]:
    return ALL_CHECKS


def get_check(check_id: CheckId | str) -> CheckDef | None:
    raw = str(check_id)
    aliases = check_rename_aliases()
    resolved = aliases.get(raw, raw)
    return _CHECK_INDEX.get(resolved)


def list_domains() -> list[str]:
    return sorted({"all", *{str(check.domain) for check in ALL_CHECKS}})


def checks_by_domain_map() -> dict[DomainId, tuple[CheckDef, ...]]:
    grouped: dict[DomainId, list[CheckDef]] = {}
    for check in ALL_CHECKS:
        key = DomainId(str(check.domain))
        grouped.setdefault(key, []).append(check)
    return {key: tuple(sorted(rows, key=lambda row: str(row.check_id))) for key, rows in grouped.items()}


def resolve_aliases() -> tuple[CheckAlias, ...]:
    expiry = check_id_alias_expiry()
    if not expiry:
        return ()
    expires_on = date.fromisoformat(expiry)
    aliases = []
    for old, new in check_rename_aliases().items():
        aliases.append(CheckAlias(old=CheckId(old), new=CheckId(new), expires_on=expires_on))
    return tuple(sorted(aliases, key=lambda item: (str(item.old), str(item.new))))


def alias_expiry_violations(today: date | None = None) -> list[str]:
    aliases = resolve_aliases()
    if not aliases:
        return []
    current = today or date.today()
    expiry = aliases[0].expires_on
    if current <= expiry:
        return []
    return [f"check id aliases expired on {expiry.isoformat()}; remove legacy id mappings"]


__all__ = [
    "ALL_CHECKS",
    "TAGS_VOCAB",
    "MARKERS_VOCAB",
    "CheckAlias",
    "check_tags",
    "check_rename_aliases",
    "checks_by_domain",
    "checks_by_domain_map",
    "get_check",
    "list_checks",
    "list_domains",
    "marker_vocabulary",
    "resolve_aliases",
    "alias_expiry_violations",
    "run_checks_for_domain",
]
