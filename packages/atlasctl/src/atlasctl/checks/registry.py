"""Checks registry SSOT accessors and loaders."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date
import json
from pathlib import Path
from typing import Mapping

from .domains import register_all
from .model import CheckDef, CheckId, DomainId, Tag
from .models import RegistryError, RegistryRecord, SuiteId, validate_tag
from ..core.runtime.paths import write_text_file

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]

REGISTRY_TOML = Path("packages/atlasctl/src/atlasctl/checks/REGISTRY.toml")
REGISTRY_JSON = Path("packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json")
REGISTRY_SCHEMA = Path("packages/atlasctl/src/atlasctl/contracts/schema/schemas/atlasctl.checks-registry.v1.schema.json")
SUITES_CATALOG_JSON = Path("packages/atlasctl/src/atlasctl/registry/suites_catalog.json")
CHECK_ID_MIGRATION_JSON = Path("configs/policy/check-id-migration.json")

ALL_CHECKS: tuple[CheckDef, ...] = register_all()

RUNTIME_REGISTRY_SOURCE = "python"
GENERATED_REGISTRY_ARTIFACTS: tuple[str, ...] = (
    "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json",
)

CHECKS_CATALOG_JSON = Path("packages/atlasctl/src/atlasctl/registry/checks_catalog.json")
CHECKS_REGISTRY_DOCS_META = Path("packages/atlasctl/docs/_meta/checks-registry.txt")


@dataclass(frozen=True)
class CompatEntry:
    old_id: str
    new_id: str
    expires_on: date


@dataclass(frozen=True)
class Registry:
    records: tuple[RegistryRecord, ...]
    suites: Mapping[str, dict[str, object]] = field(default_factory=dict)
    compat: tuple[CompatEntry, ...] = ()

    def validate(self, *, today: date | None = None) -> list[str]:
        errors: list[str] = []
        seen: set[str] = set()
        for row in self.records:
            raw_id = row.id.strip()
            if raw_id.startswith("checks_"):
                cid = str(CheckId.parse(raw_id))
            else:
                # Legacy dotted ids are tolerated during migration; dedicated
                # canonical-id checks enforce long-term cleanup.
                cid = raw_id
            if cid in seen:
                errors.append(f"duplicate check id: {cid}")
            seen.add(cid)
            try:
                DomainId.parse(row.domain)
            except ValueError as exc:
                errors.append(str(exc))
            for tag in row.tags:
                try:
                    validate_tag(tag)
                except ValueError as exc:
                    errors.append(f"{cid}: {exc}")

        current = today or date.today()
        for entry in self.compat:
            if current > entry.expires_on:
                errors.append(
                    f"compat mapping expired on {entry.expires_on.isoformat()}: {entry.old_id} -> {entry.new_id}"
                )
        return sorted(set(errors))

    def resolve_check(self, check_id: str) -> RegistryRecord:
        raw = str(check_id).strip()
        for item in self.records:
            if item.id == raw:
                return item
        mapped = self._compat_map().get(raw)
        if mapped:
            for item in self.records:
                if item.id == mapped:
                    return item
        raise RegistryError(f"unknown check id `{raw}`")

    def list_checks(self) -> tuple[RegistryRecord, ...]:
        return tuple(sorted(self.records, key=lambda row: row.id))

    def list_domains(self) -> tuple[str, ...]:
        return tuple(sorted({row.domain for row in self.records}))

    def list_suites(self) -> tuple[str, ...]:
        return tuple(sorted(self.suites.keys()))

    def expand_suite(self, suite_id: SuiteId | str) -> tuple[str, ...]:
        sid = str(SuiteId.parse(str(suite_id)))
        spec = self.suites.get(sid)
        if spec is None:
            raise RegistryError(f"unknown suite `{sid}`")
        include_checks = {str(item).strip() for item in spec.get("include_checks", []) if str(item).strip()}
        markers = {str(item).strip() for item in spec.get("markers", []) if str(item).strip()}
        exclude_markers = {str(item).strip() for item in spec.get("exclude_markers", []) if str(item).strip()}
        matched: set[str] = set()
        for row in self.records:
            tags = set(row.tags)
            if include_checks and row.id in include_checks:
                matched.add(row.id)
                continue
            if markers and tags.intersection(markers):
                if not tags.intersection(exclude_markers):
                    matched.add(row.id)
        matched.update(include_checks)
        mapped = self._compat_map()
        normalized = {mapped.get(check_id, check_id) for check_id in matched}
        return tuple(sorted(normalized))

    def _compat_map(self) -> dict[str, str]:
        return {row.old_id: row.new_id for row in self.compat}


@dataclass(frozen=True)
class RegistryEntry:
    id: str
    domain: str
    area: str
    gate: str
    owner: str
    speed: str
    groups: tuple[str, ...]
    timeout_ms: int
    module: str
    callable: str
    description: str
    severity: str
    category: str
    intent: str
    remediation_short: str
    remediation_link: str
    result_code: str
    fix_hint: str
    effects: tuple[str, ...]
    external_tools: tuple[str, ...]
    evidence: tuple[str, ...]
    writes_allowed_roots: tuple[str, ...]
    legacy_id: str | None = None


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _load_json(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RegistryError(f"invalid json object: {path.as_posix()}")
    return payload


def _load_compat_entries(repo_root: Path) -> tuple[CompatEntry, ...]:
    path = repo_root / CHECK_ID_MIGRATION_JSON
    if not path.exists():
        return ()
    payload = _load_json(path)
    expiry_raw = str(payload.get("check_ids_alias_expires_on", "")).strip()
    if not expiry_raw:
        return ()
    expiry = date.fromisoformat(expiry_raw)
    raw = payload.get("check_ids", {})
    if not isinstance(raw, dict):
        raise RegistryError("check-id migration file must define object `check_ids`")
    entries = [CompatEntry(old_id=str(old).strip(), new_id=str(new).strip(), expires_on=expiry) for old, new in raw.items()]
    return tuple(sorted(entries, key=lambda row: (row.old_id, row.new_id)))


def _load_suites(repo_root: Path) -> dict[str, dict[str, object]]:
    path = repo_root / SUITES_CATALOG_JSON
    if not path.exists():
        return {}
    payload = _load_json(path)
    rows = payload.get("suites", [])
    if not isinstance(rows, list):
        raise RegistryError("suites catalog `suites` must be a list")
    out: dict[str, dict[str, object]] = {}
    for item in rows:
        if not isinstance(item, dict):
            continue
        name = str(item.get("name", "")).strip()
        if not name:
            continue
        out[name] = item
    return dict(sorted(out.items()))


def canonical_check_id(check: CheckDef) -> str:
    return str(getattr(check, "canonical_id", "") or check.check_id)


def legacy_checks() -> tuple[CheckDef, ...]:
    return ALL_CHECKS


def load_registry_generated_json(path: Path | None = None) -> Registry:
    repo_root = _repo_root()
    target = path or (repo_root / REGISTRY_JSON)
    payload = _load_json(target)
    rows = payload.get("checks", [])
    if not isinstance(rows, list):
        raise RegistryError("registry generated json `checks` must be a list")
    records: list[RegistryRecord] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        records.append(
            RegistryRecord(
                id=str(row.get("id", "")).strip(),
                domain=str(row.get("domain", "")).strip(),
                title=str(row.get("description", "")).strip(),
                tags=tuple(str(item).strip() for item in row.get("groups", []) if str(item).strip()),
                speed=str(row.get("speed", "fast")).strip(),
                visibility="internal" if "internal" in set(row.get("groups", [])) else "public",
            )
        )
    reg = Registry(records=tuple(sorted(records, key=lambda item: item.id)), suites=_load_suites(repo_root), compat=_load_compat_entries(repo_root))
    errors = reg.validate()
    if errors:
        raise RegistryError("registry generated json invalid: " + "; ".join(errors))
    return reg


def load_registry_toml(path: Path | None = None) -> Registry:
    repo_root = _repo_root()
    target = path or (repo_root / REGISTRY_TOML)
    if not target.exists():
        raise RegistryError(f"registry toml not found: {target.as_posix()}")
    payload = tomllib.loads(target.read_text(encoding="utf-8"))
    rows = payload.get("checks", [])
    if not isinstance(rows, list):
        raise RegistryError("registry toml `checks` must be an array of tables")
    records: list[RegistryRecord] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        records.append(
            RegistryRecord(
                id=str(row.get("id", "")).strip(),
                domain=str(row.get("domain", "")).strip(),
                title=str(row.get("description", "")).strip(),
                tags=tuple(str(item).strip() for item in row.get("groups", []) if str(item).strip()),
                speed=str(row.get("speed", "fast")).strip(),
                visibility="internal" if "internal" in set(row.get("groups", [])) else "public",
            )
        )
    reg = Registry(records=tuple(sorted(records, key=lambda item: item.id)), suites=_load_suites(repo_root), compat=_load_compat_entries(repo_root))
    errors = reg.validate()
    if errors:
        raise RegistryError("registry toml invalid: " + "; ".join(errors))
    return reg


def detect_registry_drift(*, repo_root: Path | None = None) -> list[str]:
    root = repo_root or _repo_root()
    if not (root / REGISTRY_TOML).exists():
        return []
    toml_registry = load_registry_toml(root / REGISTRY_TOML)
    json_registry = load_registry_generated_json(root / REGISTRY_JSON)
    if toml_registry.list_checks() != json_registry.list_checks():
        return ["registry drift detected: REGISTRY.toml and REGISTRY.generated.json differ"]
    return []


def _entry_from_check(check: CheckDef) -> RegistryEntry:
    check_id = canonical_check_id(check)
    parts = check_id.split("_")
    area = parts[2] if len(parts) > 3 else "general"
    return RegistryEntry(
        id=check_id,
        domain=str(check.domain),
        area=area,
        gate=str(check.domain),
        owner=str(check.owners[0]) if check.owners else "platform",
        speed="slow" if bool(getattr(check, "slow", False)) else "fast",
        groups=check_tags(check),
        timeout_ms=int(getattr(check, "budget_ms", 500)),
        module=str(check.fn.__module__),
        callable=str(check.fn.__name__),
        description=str(check.description),
        severity=str(getattr(getattr(check, "severity", "error"), "value", getattr(check, "severity", "error"))),
        category=str(getattr(getattr(check, "category", "check"), "value", getattr(check, "category", "check"))),
        intent=str(getattr(check, "intent", "") or check.description),
        remediation_short=str(getattr(check, "remediation_short", "") or getattr(check, "fix_hint", "")),
        remediation_link=str(getattr(check, "remediation_link", "packages/atlasctl/docs/checks/check-id-migration-rules.md")),
        result_code=str(getattr(check, "result_code", "CHECK_GENERIC")),
        fix_hint=str(getattr(check, "fix_hint", "Review check output and apply the documented fix.")),
        effects=tuple(str(item) for item in getattr(check, "effects", ()) if str(item).strip()) or ("fs_read",),
        external_tools=tuple(str(item) for item in getattr(check, "external_tools", ()) if str(item).strip()),
        evidence=tuple(str(item) for item in getattr(check, "evidence", ()) if str(item).strip()),
        writes_allowed_roots=tuple(str(item) for item in getattr(check, "writes_allowed_roots", ("artifacts/evidence/",)) if str(item).strip()),
        legacy_id=(str(getattr(check, "legacy_check_id", "")).strip() or None),
    )


def load_registry_entries(repo_root: Path | None = None) -> tuple[RegistryEntry, ...]:
    root = repo_root or _repo_root()
    reg = load_registry_generated_json(root / REGISTRY_JSON)
    by_id: dict[str, RegistryEntry] = {}
    for check in ALL_CHECKS:
        entry = _entry_from_check(check)
        aliases = {entry.id, str(check.check_id)}
        legacy = str(getattr(check, "legacy_check_id", "") or "").strip()
        if legacy:
            aliases.add(legacy)
        for alias in aliases:
            by_id[alias] = entry
    out: list[RegistryEntry] = []
    for row in reg.list_checks():
        entry = by_id.get(row.id)
        if entry is None:
            out.append(
                RegistryEntry(
                    id=row.id,
                    domain=row.domain,
                    area="general",
                    gate=row.domain,
                    owner="platform",
                    speed=row.speed or "fast",
                    groups=tuple(sorted(set(row.tags))),
                    timeout_ms=500,
                    module="atlasctl.checks.registry",
                    callable="unknown",
                    description=row.title or row.id,
                    severity="error",
                    category="check",
                    intent=row.title or row.id,
                    remediation_short="Review check output and apply the documented fix.",
                    remediation_link="packages/atlasctl/docs/checks/check-id-migration-rules.md",
                    result_code="CHECK_GENERIC",
                    fix_hint="Review check output and apply the documented fix.",
                    effects=("fs_read",),
                    external_tools=(),
                    evidence=(),
                    writes_allowed_roots=("artifacts/evidence/",),
                    legacy_id=None,
                )
            )
            continue
        out.append(entry)
    return tuple(sorted(out, key=lambda item: item.id))


def _render_docs_meta(entries: tuple[RegistryEntry, ...]) -> str:
    lines = [
        "# atlasctl checks registry (generated)",
        "# Regenerate with: ./bin/atlasctl gen checks-registry",
        "",
        "id\tdomain\tarea\towner\tspeed\tgate\tmodule\tcallable\tdocs_link\tremediation_link",
    ]
    for row in entries:
        lines.append(
            "\t".join(
                [
                    row.id,
                    row.domain,
                    row.area,
                    row.owner,
                    row.speed,
                    row.gate,
                    row.module,
                    row.callable,
                    f"packages/atlasctl/docs/checks/index.md#{row.id}",
                    row.remediation_link,
                ]
            )
        )
    return "\n".join(lines) + "\n"


def generate_registry_json(repo_root: Path | None = None, *, check_only: bool = False) -> tuple[Path, bool]:
    root = repo_root or _repo_root()
    entries = tuple(sorted((_entry_from_check(check) for check in ALL_CHECKS), key=lambda row: row.id))
    payload = {
        "schema_version": 1,
        "kind": "atlasctl-checks-registry",
        "checks": [
            {
                "id": row.id,
                "domain": row.domain,
                "area": row.area,
                "gate": row.gate,
                "owner": row.owner,
                "speed": row.speed,
                "groups": list(row.groups),
                "timeout_ms": row.timeout_ms,
                "module": row.module,
                "callable": row.callable,
                "description": row.description,
                "severity": row.severity,
                "category": row.category,
                "intent": row.intent,
                "remediation_short": row.remediation_short,
                "remediation_link": row.remediation_link,
                "result_code": row.result_code,
                "fix_hint": row.fix_hint,
                "effects": list(row.effects),
                "external_tools": list(row.external_tools),
                "evidence": list(row.evidence),
                "writes_allowed_roots": list(row.writes_allowed_roots),
                "legacy_id": row.legacy_id,
            }
            for row in entries
        ],
    }
    catalog_payload = {
        "schema_version": 1,
        "kind": "atlasctl-checks-catalog",
        "checks": [
            {
                "id": row.id,
                "title": row.description,
                "description": row.description,
                "domain": row.domain,
                "area": row.area,
                "gate": row.gate,
                "owners": [row.owner],
                "groups": list(row.groups),
                "markers": sorted(set(row.groups)),
                "docs_link": f"packages/atlasctl/docs/checks/index.md#{row.id}",
                "remediation_link": row.remediation_link,
                "default_enabled": True,
                "impl_ref": {
                    "module": row.module,
                    "callable": row.callable,
                    "timeout_ms": row.timeout_ms,
                    "speed": row.speed,
                    "severity": row.severity,
                    "category": row.category,
                    "intent": row.intent,
                    "remediation_short": row.remediation_short,
                    "remediation_link": row.remediation_link,
                    "result_code": row.result_code,
                    "fix_hint": row.fix_hint,
                    "effects": list(row.effects),
                    "external_tools": list(row.external_tools),
                    "evidence": list(row.evidence),
                    "writes_allowed_roots": list(row.writes_allowed_roots),
                    "legacy_id": row.legacy_id,
                },
            }
            for row in entries
        ],
    }
    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    catalog_rendered = json.dumps(catalog_payload, indent=2, sort_keys=True) + "\n"
    docs_meta_rendered = _render_docs_meta(entries)
    out = root / REGISTRY_JSON
    out_catalog = root / CHECKS_CATALOG_JSON
    out_docs = root / CHECKS_REGISTRY_DOCS_META
    current = out.read_text(encoding="utf-8") if out.exists() else ""
    current_catalog = out_catalog.read_text(encoding="utf-8") if out_catalog.exists() else ""
    current_docs = out_docs.read_text(encoding="utf-8") if out_docs.exists() else ""
    changed = (current != rendered) or (current_catalog != catalog_rendered) or (current_docs != docs_meta_rendered)
    if not check_only:
        out.parent.mkdir(parents=True, exist_ok=True)
        out_catalog.parent.mkdir(parents=True, exist_ok=True)
        out_docs.parent.mkdir(parents=True, exist_ok=True)
        write_text_file(out, rendered, encoding="utf-8")
        write_text_file(out_catalog, catalog_rendered, encoding="utf-8")
        write_text_file(out_docs, docs_meta_rendered, encoding="utf-8")
    return out, changed


def registry_delta(repo_root: Path | None = None) -> dict[str, list[str]]:
    del repo_root
    entries = {entry.id for entry in (_entry_from_check(check) for check in ALL_CHECKS)}
    implemented = {canonical_check_id(check) for check in ALL_CHECKS}
    return {
        "unregistered_implementations": sorted(implemented - entries),
        "orphan_registry_entries": sorted(entries - implemented),
    }


def check_id_alias_expiry(repo_root: Path | None = None) -> str:
    root = repo_root or _repo_root()
    path = root / CHECK_ID_MIGRATION_JSON
    if not path.exists():
        return ""
    payload = _load_json(path)
    return str(payload.get("check_ids_alias_expires_on", "")).strip()


def check_id_renames(repo_root: Path | None = None) -> dict[str, str]:
    root = repo_root or _repo_root()
    path = root / CHECK_ID_MIGRATION_JSON
    if not path.exists():
        return {}
    payload = _load_json(path)
    rows = payload.get("check_ids", {})
    if not isinstance(rows, dict):
        raise RegistryError("check-id migration file must define object `check_ids`")
    return {str(k).strip(): str(v).strip() for k, v in rows.items() if str(k).strip() and str(v).strip()}


def check_rename_aliases() -> dict[str, str]:
    aliases: dict[str, str] = {}
    for check in ALL_CHECKS:
        canonical = str(check.check_id)
        legacy = str(getattr(check, "legacy_check_id", "") or "").strip()
        if legacy:
            aliases[legacy] = canonical
    aliases.update(check_id_renames())
    return dict(sorted(aliases.items()))


def get_check(check_id: CheckId | str) -> CheckDef | None:
    raw = str(check_id).strip()
    aliases = check_rename_aliases()
    resolved = aliases.get(raw, raw)
    for check in ALL_CHECKS:
        if str(check.check_id) == resolved:
            return check
    return None


def list_checks() -> tuple[CheckDef, ...]:
    return ALL_CHECKS


def check_tags(check: CheckDef) -> tuple[str, ...]:
    tags = {str(tag) for tag in getattr(check, "tags", ()) if str(tag).strip()}
    tags.add(str(check.domain))
    tags.add("slow" if bool(getattr(check, "slow", False)) else "fast")
    category = str(getattr(getattr(check, "category", "check"), "value", getattr(check, "category", "check"))).strip().lower()
    if category == "lint" or str(check.domain) in {"ops", "make", "docs", "configs"}:
        tags.add("lint")
    if "internal" not in tags and "internal-only" not in tags:
        tags.add("required")
    return tuple(sorted(tags))


def marker_vocabulary() -> tuple[str, ...]:
    return ("slow", "network", "kube", "docker", "fs-write", "git", "internal", "internal-only", "required", "fast", "lint")


TAGS_VOCAB: frozenset[Tag] = frozenset(
    Tag.parse(tag)
    for tag in {
        *(str(tag) for check in ALL_CHECKS for tag in check_tags(check)),
        *marker_vocabulary(),
        "ci",
        "dev",
        "internal",
        "lint",
        "slow",
        "fast",
    }
)


def list_domains() -> list[str]:
    return sorted({"all", *{str(check.domain) for check in ALL_CHECKS}})


def checks_by_domain() -> dict[str, list[CheckDef]]:
    grouped: dict[str, list[CheckDef]] = {}
    for check in ALL_CHECKS:
        grouped.setdefault(str(check.domain), []).append(check)
    return {key: sorted(rows, key=lambda row: str(row.check_id)) for key, rows in grouped.items()}


def checks_by_domain_map() -> dict[DomainId, tuple[CheckDef, ...]]:
    grouped: dict[DomainId, list[CheckDef]] = {}
    for check in ALL_CHECKS:
        key = DomainId(str(check.domain))
        grouped.setdefault(key, []).append(check)
    return {key: tuple(sorted(rows, key=lambda row: str(row.check_id))) for key, rows in grouped.items()}


def run_checks_for_domain(repo_root: Path, domain: DomainId | str) -> list[CheckDef]:
    del repo_root
    if str(domain) == "all":
        return list(ALL_CHECKS)
    return [check for check in ALL_CHECKS if str(check.domain) == str(domain)]


@dataclass(frozen=True)
class CheckAlias:
    old: CheckId
    new: CheckId
    expires_on: date


def resolve_aliases() -> tuple[CheckAlias, ...]:
    expiry_raw = check_id_alias_expiry()
    if not expiry_raw:
        return ()
    expiry = date.fromisoformat(expiry_raw)
    return tuple(
        sorted(
            (
                CheckAlias(
                    old=CheckId.coerce(old),
                    new=CheckId.parse(new),
                    expires_on=expiry,
                )
                for old, new in check_id_renames().items()
                if new.startswith("checks_")
            ),
            key=lambda item: (str(item.old), str(item.new)),
        )
    )


def alias_expiry_violations(today: date | None = None) -> list[str]:
    expiry_raw = check_id_alias_expiry()
    if not expiry_raw:
        return []
    expiry = date.fromisoformat(expiry_raw)
    if (today or date.today()) <= expiry:
        return []
    return [f"check id aliases expired on {expiry.isoformat()}; remove legacy id mappings"]


def runtime_registry_source() -> str:
    return RUNTIME_REGISTRY_SOURCE


def check_tree() -> dict[str, dict[str, list[str]]]:
    tree: dict[str, dict[str, list[str]]] = {}
    for check in ALL_CHECKS:
        parts = str(check.check_id).split("_")
        domain = parts[1] if len(parts) > 1 and parts[0] == "checks" else str(check.domain)
        area = parts[2] if len(parts) > 2 and parts[0] == "checks" else "general"
        tree.setdefault(domain, {}).setdefault(area, []).append(str(check.check_id))
    for domain, areas in tree.items():
        for area, ids in areas.items():
            areas[area] = sorted(ids)
        tree[domain] = dict(sorted(areas.items()))
    return dict(sorted(tree.items()))


__all__ = [
    "ALL_CHECKS",
    "CheckAlias",
    "CHECKS_CATALOG_JSON",
    "CompatEntry",
    "GENERATED_REGISTRY_ARTIFACTS",
    "CHECKS_REGISTRY_DOCS_META",
    "REGISTRY_JSON",
    "REGISTRY_TOML",
    "Registry",
    "RUNTIME_REGISTRY_SOURCE",
    "SUITES_CATALOG_JSON",
    "alias_expiry_violations",
    "check_id_alias_expiry",
    "check_id_renames",
    "check_rename_aliases",
    "check_tags",
    "check_tree",
    "canonical_check_id",
    "checks_by_domain",
    "checks_by_domain_map",
    "detect_registry_drift",
    "generate_registry_json",
    "get_check",
    "legacy_checks",
    "list_checks",
    "list_domains",
    "load_registry_entries",
    "load_registry_generated_json",
    "load_registry_toml",
    "marker_vocabulary",
    "resolve_aliases",
    "registry_delta",
    "run_checks_for_domain",
    "runtime_registry_source",
]
