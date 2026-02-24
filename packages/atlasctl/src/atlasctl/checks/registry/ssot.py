from __future__ import annotations

import importlib
import json
from datetime import date
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ..model import CheckCategory, CheckDef, CheckId, DomainId, Severity
from ..effects import CheckEffect, normalize_effect
from ..domains.configs import CHECKS as CHECKS_CONFIGS
from ..domains.docs import CHECKS as CHECKS_DOCS
from ..domains.internal import CHECKS as CHECKS_INTERNAL
from ..domains.ops import CHECKS as CHECKS_OPS
from ..domains.policies import CHECKS as CHECKS_POLICIES
from ..domains.repo import CHECKS as CHECKS_REPO
from ...core.meta.owners import load_owner_catalog
from ...core.runtime.paths import write_text_file

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    tomllib = importlib.import_module("tomli")  # type: ignore[assignment]

try:
    import jsonschema
except ModuleNotFoundError:  # pragma: no cover
    jsonschema = None  # type: ignore[assignment]


REGISTRY_TOML = Path("packages/atlasctl/src/atlasctl/checks/REGISTRY.toml")
REGISTRY_JSON = Path("packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json")
REGISTRY_SCHEMA = Path("packages/atlasctl/src/atlasctl/contracts/schema/schemas/atlasctl.checks-registry.v1.schema.json")
CHECKS_CATALOG_JSON = Path("packages/atlasctl/src/atlasctl/registry/checks_catalog.json")
CHECKS_REGISTRY_DOCS_META = Path("packages/atlasctl/docs/_meta/checks-registry.txt")
CHECKS_CATALOG_SCHEMA = Path("packages/atlasctl/src/atlasctl/contracts/schema/schemas/atlasctl.checks-catalog.v1.schema.json")
RENAMES_JSON = Path("configs/policy/target-renames.json")
CHECK_ID_MIGRATION_JSON = Path("configs/policy/check-id-migration.json")
FILENAME_ALLOWLIST_JSON = Path("configs/policy/check-filename-allowlist.json")
TRANSITION_ALLOWLIST_JSON = Path("configs/policy/checks-registry-transition.json")
CHECK_CATEGORY_ENUM = {"lint", "check"}
CHECK_DOMAIN_ENUM = {"checks", "configs", "contracts", "docker", "docs", "license", "make", "ops", "policies", "python", "repo"}
CHECK_EFFECT_ENUM = {CheckEffect.FS_READ.value, CheckEffect.FS_WRITE.value, CheckEffect.SUBPROCESS.value, CheckEffect.GIT.value, CheckEffect.NETWORK.value}
LINT_DOMAINS = {"configs", "docs", "make", "ops"}
DOMAIN_MAX_PY_FILES = 140


def normalize_category(raw: str, *, domain: str, groups: tuple[str, ...]) -> str:
    value = str(raw).strip().lower()
    if value in CHECK_CATEGORY_ENUM:
        return value
    if "lint" in {g.strip().lower() for g in groups}:
        return "lint"
    if domain in LINT_DOMAINS:
        return "lint"
    return "check"


def _default_groups_for_check(check: CheckDef) -> tuple[str, ...]:
    tags = {str(item).strip() for item in check.tags if str(item).strip()}
    tags.add(str(check.domain))
    tags.add("slow" if bool(check.slow) else "fast")
    if "internal" not in tags and "internal-only" not in tags:
        tags.add("required")
    return tuple(sorted(tags))


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
    severity: str = "error"
    category: str = "check"
    intent: str = ""
    remediation_short: str = "Review check output and apply the documented fix."
    remediation_link: str = "packages/atlasctl/docs/checks/check-id-migration-rules.md"
    result_code: str = "CHECK_GENERIC"
    fix_hint: str = "Review check output and apply the documented fix."
    effects: tuple[str, ...] = ()
    external_tools: tuple[str, ...] = ()
    evidence: tuple[str, ...] = ()
    writes_allowed_roots: tuple[str, ...] = ("artifacts/evidence/",)
    legacy_id: str | None = None


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[6]


def _load_schema(repo_root: Path) -> dict[str, object]:
    return json.loads((repo_root / REGISTRY_SCHEMA).read_text(encoding="utf-8"))


def _parse_toml(repo_root: Path) -> list[RegistryEntry]:
    raw = tomllib.loads((repo_root / REGISTRY_TOML).read_text(encoding="utf-8"))
    rows = raw.get("checks", [])
    if not isinstance(rows, list):
        raise ValueError("checks registry must define [[checks]] array")
    entries: list[RegistryEntry] = []
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("invalid checks registry row")
        entries.append(
            RegistryEntry(
                id=str(row.get("id", "")).strip(),
                domain=str(row.get("domain", "")).strip(),
                area=str(row.get("area", "")).strip(),
                gate=str(row.get("gate", row.get("domain", ""))).strip(),
                owner=str(row.get("owner", "")).strip(),
                speed=str(row.get("speed", "")).strip(),
                groups=tuple(str(x).strip() for x in row.get("groups", []) if str(x).strip()),
                timeout_ms=int(row.get("timeout_ms", 0)),
                module=str(row.get("module", "")).strip(),
                callable=str(row.get("callable", "")).strip(),
                description=str(row.get("description", "")).strip(),
                severity=str(row.get("severity", "error")).strip(),
                category=normalize_category(str(row.get("category", "check")), domain=str(row.get("domain", "")).strip(), groups=tuple(str(x).strip() for x in row.get("groups", []) if str(x).strip())),
                intent=str(row.get("intent", row.get("description", ""))).strip(),
                remediation_short=str(row.get("remediation_short", row.get("fix_hint", "Review check output and apply the documented fix."))).strip(),
                remediation_link=str(row.get("remediation_link", "packages/atlasctl/docs/checks/check-id-migration-rules.md")).strip(),
                result_code=str(row.get("result_code", "CHECK_GENERIC")).strip(),
                fix_hint=str(row.get("fix_hint", "Review check output and apply the documented fix.")).strip(),
                effects=tuple(
                    normalize_effect(str(x))
                    for x in row.get("effects", [CheckEffect.FS_READ.value])
                    if str(x).strip()
                )
                or (CheckEffect.FS_READ.value,),
                external_tools=tuple(str(x).strip() for x in row.get("external_tools", []) if str(x).strip()),
                evidence=tuple(str(x).strip() for x in row.get("evidence", []) if str(x).strip()),
                writes_allowed_roots=tuple(str(x).strip() for x in row.get("writes_allowed_roots", ["artifacts/evidence/"]) if str(x).strip()),
                legacy_id=(str(row.get("legacy_id")).strip() if row.get("legacy_id") is not None else None),
            )
        )
    return entries


def _entry_as_dict(entry: RegistryEntry) -> dict[str, Any]:
    return {
        "id": entry.id,
        "domain": entry.domain,
        "area": entry.area,
        "gate": entry.gate,
        "owner": entry.owner,
        "speed": entry.speed,
        "groups": list(entry.groups),
        "timeout_ms": entry.timeout_ms,
        "module": entry.module,
        "callable": entry.callable,
        "description": entry.description,
        "severity": entry.severity,
        "category": entry.category,
        "intent": entry.intent,
        "remediation_short": entry.remediation_short,
        "remediation_link": entry.remediation_link,
        "result_code": entry.result_code,
        "fix_hint": entry.fix_hint,
        "effects": list(entry.effects),
        "external_tools": list(entry.external_tools),
        "evidence": list(entry.evidence),
        "writes_allowed_roots": list(entry.writes_allowed_roots),
        "legacy_id": entry.legacy_id,
    }


def _validate_entries(entries: list[RegistryEntry]) -> None:
    errors: list[str] = []
    seen: set[str] = set()
    valid_owners = set(load_owner_catalog(_repo_root()).owners)
    allowlist: set[str] = set()
    allowlist_payload_path = _repo_root() / FILENAME_ALLOWLIST_JSON
    if allowlist_payload_path.exists():
        payload = json.loads(allowlist_payload_path.read_text(encoding="utf-8"))
        allowlist = {str(name) for name in payload.get("allowlist", [])}
    legacy_map = legacy_check_by_id()
    for idx, e in enumerate(entries, start=1):
        if e.id in seen:
            errors.append(f"duplicate id: {e.id}")
        seen.add(e.id)
        if not e.id.startswith(f"checks_{e.domain}_"):
            errors.append(f"{e.id}: id/domain mismatch")
        import re
        if re.match(r"^checks_[a-z0-9]+_[a-z0-9]+_[a-z0-9_]+$", e.id) is None:
            errors.append(f"{e.id}: id must match checks_<domain>_<area>_<name>")
        if e.domain not in CHECK_DOMAIN_ENUM:
            errors.append(f"{e.id}: domain must be one of {sorted(CHECK_DOMAIN_ENUM)}")
        if e.category not in CHECK_CATEGORY_ENUM:
            errors.append(f"{e.id}: category must be one of {sorted(CHECK_CATEGORY_ENUM)}")
        if not e.intent or len(e.intent.split()) < 3:
            errors.append(f"{e.id}: intent must be a meaningful one-sentence description")
        if not e.remediation_short:
            errors.append(f"{e.id}: remediation_short is required (or explicit `none`)")
        if not e.remediation_link:
            errors.append(f"{e.id}: remediation_link is required (or explicit `none`)")
        if not e.result_code:
            errors.append(f"{e.id}: result_code is required")
        if e.fix_hint.strip() == "Review check output and apply the documented fix.":
            errors.append(f"{e.id}: fix_hint must be actionable, not boilerplate")
        unknown_effects = sorted(set(e.effects).difference(CHECK_EFFECT_ENUM))
        if unknown_effects:
            errors.append(f"{e.id}: unknown effects {unknown_effects}; allowed {sorted(CHECK_EFFECT_ENUM)}")
        if "lint" in {group.lower() for group in e.groups}:
            errors.append(f"{e.id}: `lint` marker in groups is forbidden; use category=lint")
        if e.speed not in {"fast", "slow", "nightly"}:
            errors.append(f"{e.id}: speed must be fast|slow|nightly")
        if not e.groups:
            errors.append(f"{e.id}: groups must not be empty")
        if not e.owner:
            errors.append(f"{e.id}: owner must not be empty")
        if e.owner not in valid_owners:
            errors.append(f"{e.id}: owner `{e.owner}` not present in configs/meta/owners.json")
        if e.timeout_ms < 50:
            errors.append(f"{e.id}: timeout_ms must be >= 50ms")
        if e.speed in {"slow", "nightly"} and e.timeout_ms < 2000:
            errors.append(f"{e.id}: slow/nightly checks must have timeout_ms >= 2000")
        write_roots = tuple(e.writes_allowed_roots)
        default_write_roots = ("artifacts/evidence/",)
        if CheckEffect.FS_WRITE.value in e.effects and not write_roots:
            errors.append(f"{e.id}: fs_write checks must declare writes_allowed_roots")
        if write_roots != default_write_roots and "fs-write" not in e.groups:
            errors.append(f"{e.id}: non-default writes_allowed_roots require `fs-write` group marker")
        for root in write_roots:
            if not (root.startswith("artifacts/evidence/") or root.startswith("artifacts/reports/") or root.startswith("artifacts/isolate/")):
                errors.append(f"{e.id}: writes_allowed_roots path not allowed: {root}")
        try:
            mod = importlib.import_module(e.module)
        except Exception as exc:  # pragma: no cover
            errors.append(f"{e.id}: module import failed `{e.module}` ({exc})")
            continue
        fn = getattr(mod, e.callable, None)
        if e.callable == "CHECKS":
            exported = getattr(mod, "CHECKS", None)
            if not isinstance(exported, (list, tuple)):
                errors.append(f"{e.id}: `{e.module}:CHECKS` must export a list/tuple")
            else:
                matched = False
                for item in exported:
                    check_id = getattr(item, "check_id", "") or getattr(getattr(item, "__atlasctl_check_meta__", None), "check_id", "")
                    if str(check_id) == e.id:
                        matched = True
                        break
                if not matched:
                    errors.append(f"{e.id}: CHECKS export does not contain matching check_id")
        elif (fn is None or not callable(fn)) and e.id not in legacy_map:
            errors.append(f"{e.id}: callable not found `{e.module}:{e.callable}`")
        source = Path(getattr(mod, "__file__", ""))
        if source and source.name and source.name.endswith(".py") and source.name == "__init__.py" and e.id not in legacy_map:
            errors.append(f"{e.id}: check module must not be __init__.py")
        if source and source.exists():
            text = source.read_text(encoding="utf-8", errors="ignore")
            if text.count("@check(") > 1 and e.callable != "CHECKS":
                errors.append(f"{e.id}: module defines multiple checks; registry callable must be CHECKS")
        if e.domain not in e.id.split("_"):
            errors.append(f"{e.id}: domain segment missing from id")
        legacy = legacy_map.get(e.id)
        if legacy is not None:
            expected_speed = "slow" if bool(legacy.slow) else "fast"
            if e.speed not in {expected_speed, "nightly"}:
                errors.append(f"{e.id}: speed mismatch; expected `{expected_speed}` (or nightly) from implementation")
        if idx > 1 and entries[idx - 2].id > e.id:
            errors.append("registry entries must be sorted by id")

    source_root = _repo_root() / "packages/atlasctl/src"
    domains_root = source_root / "atlasctl/checks/domains"
    if domains_root.exists():
        for domain_dir in sorted(path for path in domains_root.iterdir() if path.is_dir()):
            count = sum(1 for p in domain_dir.rglob("*.py") if p.is_file())
            if count > DOMAIN_MAX_PY_FILES:
                errors.append(f"domain `{domain_dir.name}` exceeds python file budget: {count} > {DOMAIN_MAX_PY_FILES}")
    for entry in entries:
        module_file = source_root / (entry.module.replace(".", "/") + ".py")
        package_init = source_root / (entry.module.replace(".", "/") + "/__init__.py")
        if not module_file.exists() and not package_init.exists():
            errors.append(f"{entry.id}: implementation module file missing for `{entry.module}`")
    if errors:
        raise ValueError("checks registry invariants failed: " + "; ".join(sorted(set(errors))))


def load_registry_entries(repo_root: Path | None = None) -> tuple[RegistryEntry, ...]:
    root = repo_root or _repo_root()
    payload = json.loads((root / CHECKS_CATALOG_JSON).read_text(encoding="utf-8"))
    rows = payload.get("checks", [])
    if not isinstance(rows, list):
        raise ValueError("checks registry generated payload missing `checks` list")
    entries = [
        RegistryEntry(
            id=str(r["id"]),
            domain=str(r["domain"]),
            area=str(r["area"]),
            gate=str(r.get("gate", r.get("domain", ""))),
            owner=str((r.get("owners") or [""])[0]),
            speed=str(r.get("impl_ref", {}).get("speed", "fast")),
            groups=tuple(str(x) for x in r.get("groups", [])),
            timeout_ms=int(r.get("impl_ref", {}).get("timeout_ms", 2000)),
            module=str(r.get("impl_ref", {}).get("module", "")),
            callable=str(r.get("impl_ref", {}).get("callable", "")),
            description=str(r["description"]),
            severity=str(r.get("impl_ref", {}).get("severity", "error")),
            category=normalize_category(
                str(r.get("impl_ref", {}).get("category", "check")),
                domain=str(r["domain"]),
                groups=tuple(str(x) for x in r.get("groups", [])),
            ),
            intent=str(r.get("impl_ref", {}).get("intent", r.get("description", ""))),
            remediation_short=str(r.get("impl_ref", {}).get("remediation_short", r.get("impl_ref", {}).get("fix_hint", "Review check output and apply the documented fix."))),
            remediation_link=str(r.get("impl_ref", {}).get("remediation_link", "packages/atlasctl/docs/checks/check-id-migration-rules.md")),
            result_code=str(r.get("impl_ref", {}).get("result_code", "CHECK_GENERIC")),
            fix_hint=str(r.get("impl_ref", {}).get("fix_hint", "Review check output and apply the documented fix.")),
            effects=tuple(
                normalize_effect(str(x))
                for x in r.get("impl_ref", {}).get("effects", [CheckEffect.FS_READ.value])
                if str(x).strip()
            )
            or (CheckEffect.FS_READ.value,),
            external_tools=tuple(str(x) for x in r.get("impl_ref", {}).get("external_tools", [])),
            evidence=tuple(str(x) for x in r.get("impl_ref", {}).get("evidence", [])),
            writes_allowed_roots=tuple(str(x) for x in r.get("impl_ref", {}).get("writes_allowed_roots", ["artifacts/evidence/"])),
            legacy_id=(str(r.get("impl_ref", {}).get("legacy_id")) if r.get("impl_ref", {}).get("legacy_id") else None),
        )
        for r in rows
    ]
    _validate_entries(entries)
    return tuple(entries)


def _to_catalog_entry(entry: RegistryEntry) -> dict[str, Any]:
    markers = {
        *entry.groups,
        entry.domain,
        entry.speed,
        *(("required",) if "internal" not in entry.groups and "internal-only" not in entry.groups else ()),
    }
    if "network" in entry.id:
        markers.add("network")
    if entry.domain == "docker" or "docker" in entry.id:
        markers.add("docker")
    if entry.domain == "ops" or "kube" in entry.id or "k8s" in entry.id:
        markers.add("kube")
    if "write" in entry.effects:
        markers.add("fs-write")
    if "git" in entry.id:
        markers.add("git")
    return {
        "id": entry.id,
        "title": entry.description,
        "description": entry.description,
        "domain": entry.domain,
        "area": entry.area,
        "gate": entry.gate,
        "owners": [entry.owner],
        "groups": list(entry.groups),
        "markers": sorted(markers),
        "docs_link": f"packages/atlasctl/docs/checks/index.md#{entry.id}",
        "remediation_link": entry.remediation_link,
        "default_enabled": True,
        "impl_ref": {
            "module": entry.module,
            "callable": entry.callable,
            "timeout_ms": entry.timeout_ms,
            "speed": entry.speed,
            "severity": entry.severity,
            "category": entry.category,
            "intent": entry.intent,
            "remediation_short": entry.remediation_short,
            "remediation_link": entry.remediation_link,
            "result_code": entry.result_code,
            "fix_hint": entry.fix_hint,
            "effects": list(entry.effects),
            "external_tools": list(entry.external_tools),
            "evidence": list(entry.evidence),
            "writes_allowed_roots": list(entry.writes_allowed_roots),
            "legacy_id": entry.legacy_id,
        },
    }


def _render_docs_meta(catalog_payload: dict[str, Any]) -> str:
    lines = [
        "# atlasctl checks registry (generated)",
        "# Regenerate with: ./bin/atlasctl gen checks-registry",
        "",
        "id\tdomain\tarea\towner\tspeed\tgate\tmodule\tcallable\tdocs_link\tremediation_link",
    ]
    for row in catalog_payload.get("checks", []):
        impl_ref = row.get("impl_ref", {})
        owners = row.get("owners", [])
        owner = str(owners[0]) if isinstance(owners, list) and owners else ""
        lines.append(
            "\t".join(
                [
                    str(row.get("id", "")),
                    str(row.get("domain", "")),
                    str(row.get("area", "")),
                    owner,
                    str(impl_ref.get("speed", "")),
                    str(row.get("gate", "")),
                    str(impl_ref.get("module", "")),
                    str(impl_ref.get("callable", "")),
                    str(row.get("docs_link", "")),
                    str(row.get("remediation_link", "")),
                ]
            )
        )
    return "\n".join(lines) + "\n"


def generate_registry_json(repo_root: Path | None = None, *, check_only: bool = False) -> tuple[Path, bool]:
    root = repo_root or _repo_root()
    checks = sorted(legacy_checks(), key=lambda check: str(check.check_id))
    entries = [
        RegistryEntry(
            id=str(row["id"]),
            domain=str(row["domain"]),
            area=str(row["area"]),
            gate=str(row["gate"]),
            owner=str(row["owner"]),
            speed=str(row["speed"]),
            groups=tuple(str(item) for item in row.get("groups", []) if str(item).strip()),
            timeout_ms=int(row["timeout_ms"]),
            module=str(row["module"]),
            callable=str(row["callable"]),
            description=str(row["description"]),
            severity=str(row.get("severity", "error")),
            category=normalize_category(str(row.get("category", "check")), domain=str(row["domain"]), groups=tuple(str(item) for item in row.get("groups", []))),
            intent=str(row.get("intent", row.get("description", ""))),
            remediation_short=str(row.get("remediation_short", row.get("fix_hint", "Review check output and apply the documented fix."))),
            remediation_link=str(row.get("remediation_link", "packages/atlasctl/docs/checks/check-id-migration-rules.md")),
            result_code=str(row.get("result_code", "CHECK_GENERIC")),
            fix_hint=str(row.get("fix_hint", "Review check output and apply the documented fix.")),
            effects=tuple(
                normalize_effect(str(item))
                for item in row.get("effects", [CheckEffect.FS_READ.value])
                if str(item).strip()
            )
            or (CheckEffect.FS_READ.value,),
            external_tools=tuple(str(item) for item in row.get("external_tools", []) if str(item).strip()),
            evidence=tuple(str(item) for item in row.get("evidence", []) if str(item).strip()),
            writes_allowed_roots=tuple(str(item) for item in row.get("writes_allowed_roots", ["artifacts/evidence/"]) if str(item).strip()),
            legacy_id=(str(row.get("legacy_id")).strip() if row.get("legacy_id") is not None else None),
        )
        for row in (toml_entry_from_check(check, groups=_default_groups_for_check(check)) for check in checks)
    ]
    entries = sorted(entries, key=lambda entry: entry.id)
    _validate_entries(entries)
    legacy = {canonical_check_id(check) for check in checks}
    registered = {entry.id for entry in entries}
    missing_registry = sorted(legacy - registered)
    missing_impl = sorted(registered - legacy)
    transition = _load_transition_allowlist(root)
    allow = set(transition["allowlist"])
    if transition["active"]:
        missing_registry = sorted(item for item in missing_registry if item not in allow)
    if missing_registry or missing_impl:
        errors: list[str] = []
        if missing_registry:
            errors.append("unregistered check implementations: " + ", ".join(missing_registry))
        if missing_impl:
            errors.append("registry entries missing implementations: " + ", ".join(missing_impl))
        raise ValueError("; ".join(errors))
    payload = {
        "schema_version": 1,
        "kind": "atlasctl-checks-registry",
        "checks": [_entry_as_dict(e) for e in entries],
    }
    catalog_payload = {
        "schema_version": 1,
        "kind": "atlasctl-checks-catalog",
        "checks": [_to_catalog_entry(e) for e in entries],
    }
    if jsonschema is not None:
        schema = _load_schema(root)
        jsonschema.validate(payload, schema)
        catalog_schema = json.loads((root / CHECKS_CATALOG_SCHEMA).read_text(encoding="utf-8"))
        jsonschema.validate(catalog_payload, catalog_schema)
    out = root / REGISTRY_JSON
    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    catalog_out = root / CHECKS_CATALOG_JSON
    catalog_rendered = json.dumps(catalog_payload, indent=2, sort_keys=True) + "\n"
    docs_meta_out = root / CHECKS_REGISTRY_DOCS_META
    docs_meta_rendered = _render_docs_meta(catalog_payload)
    current = out.read_text(encoding="utf-8") if out.exists() else ""
    catalog_current = catalog_out.read_text(encoding="utf-8") if catalog_out.exists() else ""
    docs_meta_current = docs_meta_out.read_text(encoding="utf-8") if docs_meta_out.exists() else ""
    changed = current != rendered
    catalog_changed = catalog_current != catalog_rendered
    docs_meta_changed = docs_meta_current != docs_meta_rendered
    if not check_only and changed:
        out.parent.mkdir(parents=True, exist_ok=True)
        write_text_file(out, rendered, encoding="utf-8")
    if not check_only and catalog_changed:
        catalog_out.parent.mkdir(parents=True, exist_ok=True)
        write_text_file(catalog_out, catalog_rendered, encoding="utf-8")
    if not check_only and docs_meta_changed:
        docs_meta_out.parent.mkdir(parents=True, exist_ok=True)
        write_text_file(docs_meta_out, docs_meta_rendered, encoding="utf-8")
    return out, (changed or catalog_changed or docs_meta_changed)


def registry_delta(repo_root: Path | None = None) -> dict[str, list[str]]:
    entries = [
        RegistryEntry(
            id=str(row["id"]),
            domain=str(row["domain"]),
            area=str(row["area"]),
            gate=str(row["gate"]),
            owner=str(row["owner"]),
            speed=str(row["speed"]),
            groups=tuple(str(item) for item in row.get("groups", []) if str(item).strip()),
            timeout_ms=int(row["timeout_ms"]),
            module=str(row["module"]),
            callable=str(row["callable"]),
            description=str(row["description"]),
        )
        for row in (toml_entry_from_check(check, groups=_default_groups_for_check(check)) for check in legacy_checks())
    ]
    registered = {entry.id for entry in entries}
    implemented = {canonical_check_id(check) for check in legacy_checks()}
    return {
        "unregistered_implementations": sorted(implemented - registered),
        "orphan_registry_entries": sorted(registered - implemented),
    }


def _load_transition_allowlist(repo_root: Path) -> dict[str, object]:
    path = repo_root / TRANSITION_ALLOWLIST_JSON
    if not path.exists():
        return {"active": False, "allowlist": []}
    payload = json.loads(path.read_text(encoding="utf-8"))
    expiry = str(payload.get("allow_unregistered_until", "")).strip()
    allowlist = [str(item).strip() for item in payload.get("allowlist", []) if str(item).strip()]
    active = False
    try:
        active = bool(expiry) and date.today() <= date.fromisoformat(expiry)
    except ValueError:
        active = False
    return {"active": active, "allowlist": allowlist}


def toml_entry_from_check(check: CheckDef, *, groups: tuple[str, ...]) -> dict[str, Any]:
    canonical_id = canonical_check_id(check)
    segments = canonical_id.split("_")
    area = segments[2] if len(segments) > 3 else "general"
    owner = check.owners[0] if check.owners else "platform"
    return {
        "id": canonical_id,
        "domain": check.domain,
        "area": area,
        "gate": check.domain,
        "owner": owner,
        "speed": "slow" if check.slow else "fast",
        "groups": list(groups),
        "timeout_ms": max(check.budget_ms, 2000) if check.slow else check.budget_ms,
        "module": check.fn.__module__,
        "callable": check.fn.__name__,
        "description": check.description,
        "severity": check.severity.value,
        "category": check.category.value,
        "intent": check.intent or check.description,
        "remediation_short": check.remediation_short or check.fix_hint,
        "remediation_link": check.remediation_link,
        "result_code": check.result_code,
        "fix_hint": check.fix_hint,
        "effects": list(check.effects),
        "external_tools": list(check.external_tools),
        "evidence": list(check.evidence),
        "writes_allowed_roots": list(check.writes_allowed_roots),
        "legacy_id": check.check_id if canonical_id != check.check_id else check.legacy_check_id,
    }


def write_registry_toml(repo_root: Path, rows: list[dict[str, Any]]) -> Path:
    out = repo_root / REGISTRY_TOML
    lines = [
        "# This file is the SSOT for atlasctl checks. Keep entries sorted by `id`.",
        "# Regenerate with: ./bin/atlasctl gen checks-registry",
        "",
        'schema = "atlasctl.checks-registry.v1"',
        "",
    ]
    def q(s: str) -> str:
        return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'
    for row in rows:
        lines.append("[[checks]]")
        for key in (
            "id",
            "domain",
            "area",
            "gate",
            "owner",
            "speed",
            "timeout_ms",
            "module",
            "callable",
            "description",
            "severity",
            "category",
            "intent",
            "remediation_short",
            "remediation_link",
            "result_code",
            "fix_hint",
            "legacy_id",
        ):
            value = row.get(key)
            if value in (None, ""):
                continue
            if isinstance(value, int):
                lines.append(f"{key} = {value}")
            else:
                lines.append(f"{key} = {q(str(value))}")
        for key in ("groups", "effects", "external_tools", "evidence", "writes_allowed_roots"):
            arr = [str(v) for v in row.get(key, []) if str(v).strip()]
            rendered = ", ".join(q(v) for v in arr)
            lines.append(f"{key} = [{rendered}]")
        lines.append("")
    out.parent.mkdir(parents=True, exist_ok=True)
    write_text_file(out, "\n".join(lines), encoding="utf-8")
    return out


def legacy_checks() -> tuple[CheckDef, ...]:
    return (
        *CHECKS_REPO,
        *CHECKS_DOCS,
        *CHECKS_OPS,
        *CHECKS_CONFIGS,
        *CHECKS_POLICIES,
        *CHECKS_INTERNAL,
    )


def _rename_overrides(repo_root: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    path = repo_root / RENAMES_JSON
    if path.exists():
        payload = json.loads(path.read_text(encoding="utf-8"))
        rows = payload.get("check_ids", {})
        if isinstance(rows, dict):
            out.update({str(old): str(new) for old, new in rows.items()})
    migration = repo_root / CHECK_ID_MIGRATION_JSON
    if migration.exists():
        payload = json.loads(migration.read_text(encoding="utf-8"))
        rows = payload.get("check_ids", {})
        if isinstance(rows, dict):
            out.update({str(old): str(new) for old, new in rows.items()})
    return out


def check_id_renames(repo_root: Path | None = None) -> dict[str, str]:
    root = repo_root or _repo_root()
    return dict(sorted(_rename_overrides(root).items()))


def check_id_alias_expiry(repo_root: Path | None = None) -> str | None:
    root = repo_root or _repo_root()
    for rel in (CHECK_ID_MIGRATION_JSON, RENAMES_JSON):
        path = root / rel
        if not path.exists():
            continue
        payload = json.loads(path.read_text(encoding="utf-8"))
        expiry = str(payload.get("check_ids_alias_expires_on", "")).strip()
        if expiry:
            return expiry
    return None


def canonical_check_id(check: CheckDef) -> str:
    overrides = _rename_overrides(_repo_root())
    if check.check_id in overrides:
        return _normalize_checks_id(overrides[check.check_id], check.domain)
    raw = check.check_id.replace(".", "_").replace("-", "_").lower()
    raw = "_".join(token for token in raw.split("_") if token)
    if raw.startswith("checks_"):
        return _normalize_checks_id(raw, check.domain)
    if raw.startswith(f"{check.domain}_"):
        payload = raw[len(check.domain) + 1 :]
        return _normalize_checks_id(f"checks_{check.domain}_{payload}", check.domain)
    payload = raw
    return _normalize_checks_id(f"checks_{check.domain}_{payload}", check.domain)


def _normalize_checks_id(check_id: CheckId | str, domain: DomainId | str) -> str:
    token = str(check_id).strip().replace(".", "_").replace("-", "_").lower()
    domain_token = str(domain)
    token = "_".join(part for part in token.split("_") if part)
    if not token.startswith("checks_"):
        if token.startswith(f"{domain_token}_"):
            token = f"checks_{token}"
        else:
            token = f"checks_{domain_token}_{token}"
    parts = token.split("_")
    if len(parts) < 3:
        token = f"checks_{domain_token}_{'_'.join(parts[1:]) if len(parts) > 1 else token}"
    elif parts[1] != domain_token:
        token = "checks_" + domain_token + "_" + "_".join(parts[1:])
    return token


def legacy_check_by_id() -> dict[str, CheckDef]:
    return {canonical_check_id(check): check for check in legacy_checks()}
