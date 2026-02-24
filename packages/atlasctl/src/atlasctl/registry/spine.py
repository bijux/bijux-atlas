from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ..checks.registry import REGISTRY_SCHEMA, load_registry_entries
from ..cli.surface_registry import command_registry
from ..core.meta.owners import load_owner_catalog

from ..runtime.capabilities import capabilities_for_command
from ..commands.policies.runtime.culprits import load_budgets
from .catalogs import load_suites_catalog

try:
    import jsonschema
except ModuleNotFoundError:  # pragma: no cover
    jsonschema = None  # type: ignore[assignment]


REGISTRY_SPINE_GENERATED_JSON = Path("packages/atlasctl/src/atlasctl/registry/registry_spine.generated.json")
REGISTRY_SPINE_SCHEMA = Path("packages/atlasctl/src/atlasctl/contracts/schema/schemas/atlasctl.registry-spine.v1.schema.json")


@dataclass(frozen=True)
class BudgetModel:
    defaults: dict[str, int]
    rules_count: int
    exceptions_count: int


@dataclass(frozen=True)
class CapabilityModel:
    subject_kind: str
    subject_name: str
    effects: tuple[str, ...]
    tools: tuple[str, ...]
    write_roots: tuple[str, ...]
    network: bool


@dataclass(frozen=True)
class RegistryCommand:
    group: str
    name: str
    owner: str
    help_text: str
    tags: tuple[str, ...]
    aliases: tuple[str, ...]
    internal: bool
    stable: bool


@dataclass(frozen=True)
class RegistryCheck:
    check_id: str
    domain: str
    category: str
    severity: str
    owner: str
    description: str
    tags: tuple[str, ...]
    suite_membership: tuple[str, ...]
    effects: tuple[str, ...]


@dataclass(frozen=True)
class RegistrySuite:
    name: str
    markers: tuple[str, ...]
    include_checks: tuple[str, ...]
    internal: bool


@dataclass(frozen=True)
class Registry:
    version: int
    commands: tuple[RegistryCommand, ...]
    checks: tuple[RegistryCheck, ...]
    suites: tuple[RegistrySuite, ...]
    owners: tuple[str, ...]
    budgets: BudgetModel
    capabilities: tuple[CapabilityModel, ...]

    def select_checks(
        self,
        *,
        domain: str | None = None,
        tags: tuple[str, ...] = (),
        severity: str | None = None,
        suite: str | None = None,
    ) -> tuple[RegistryCheck, ...]:
        required_tags = set(tags)
        out: list[RegistryCheck] = []
        for item in self.checks:
            if domain and item.domain != domain:
                continue
            if severity and item.severity != severity:
                continue
            if suite and suite not in item.suite_membership:
                continue
            if required_tags and required_tags.difference(item.tags):
                continue
            out.append(item)
        return tuple(out)

    def select_commands(self, *, group: str | None = None, tags: tuple[str, ...] = ()) -> tuple[RegistryCommand, ...]:
        required_tags = set(tags)
        out: list[RegistryCommand] = []
        for item in self.commands:
            if group and item.group != group:
                continue
            if required_tags and required_tags.difference(item.tags):
                continue
            out.append(item)
        return tuple(out)

    def as_generated_payload(self) -> dict[str, Any]:
        return {
            "schema_name": "atlasctl.registry-spine.v1",
            "schema_version": 1,
            "version": self.version,
            "commands": [
                {
                    "group": c.group,
                    "name": c.name,
                    "owner": c.owner,
                    "help": c.help_text,
                    "tags": list(c.tags),
                    "aliases": list(c.aliases),
                    "internal": c.internal,
                    "stable": c.stable,
                }
                for c in self.commands
            ],
            "checks": [
                {
                    "id": c.check_id,
                    "domain": c.domain,
                    "category": c.category,
                    "severity": c.severity,
                    "owner": c.owner,
                    "description": c.description,
                    "tags": list(c.tags),
                    "suite_membership": list(c.suite_membership),
                    "effects": list(c.effects),
                }
                for c in self.checks
            ],
            "suites": [
                {
                    "name": s.name,
                    "markers": list(s.markers),
                    "include_checks": list(s.include_checks),
                    "internal": s.internal,
                }
                for s in self.suites
            ],
            "owners": list(self.owners),
            "budgets": {
                "defaults": self.budgets.defaults,
                "rules_count": self.budgets.rules_count,
                "exceptions_count": self.budgets.exceptions_count,
            },
            "capabilities": [
                {
                    "subject_kind": c.subject_kind,
                    "subject_name": c.subject_name,
                    "effects": list(c.effects),
                    "tools": list(c.tools),
                    "write_roots": list(c.write_roots),
                    "network": c.network,
                }
                for c in self.capabilities
            ],
        }


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _load_registry_schema_input(repo_root: Path) -> None:
    if jsonschema is None:
        return
    schema = json.loads((repo_root / REGISTRY_SCHEMA).read_text(encoding="utf-8"))
    raw = json.loads((repo_root / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json").read_text(encoding="utf-8"))
    jsonschema.validate(raw, schema)


def load_registry(repo_root: Path | None = None) -> Registry:
    root = repo_root or _repo_root()
    _load_registry_schema_input(root)

    owner_catalog = load_owner_catalog(root)
    check_entries = load_registry_entries(root)
    suites_payload = load_suites_catalog(root)
    suite_rows = suites_payload.get("suites", []) if isinstance(suites_payload, dict) else []

    suites: list[RegistrySuite] = []
    suite_by_check: dict[str, set[str]] = {}
    for row in suite_rows:
        if not isinstance(row, dict):
            continue
        suite = RegistrySuite(
            name=str(row.get("name", "")).strip(),
            markers=tuple(sorted(str(x).strip() for x in row.get("markers", []) if str(x).strip())),
            include_checks=tuple(sorted(str(x).strip() for x in row.get("include_checks", []) if str(x).strip())),
            internal=bool(row.get("internal", False)),
        )
        if not suite.name:
            continue
        suites.append(suite)
        for cid in suite.include_checks:
            suite_by_check.setdefault(cid, set()).add(suite.name)

    checks = [
        RegistryCheck(
            check_id=e.id,
            domain=e.domain,
            category=e.category,
            severity=e.severity,
            owner=e.owner,
            description=e.description,
            tags=tuple(sorted(set(e.groups))),
            suite_membership=tuple(sorted(suite_by_check.get(e.id, set()))),
            effects=e.effects,
        )
        for e in check_entries
    ]
    checks.sort(key=lambda c: c.check_id)

    commands: list[RegistryCommand] = []
    capabilities: list[CapabilityModel] = []
    for spec in command_registry():
        group = spec.name.split("-", 1)[0]
        tags = tuple(sorted(set((group, *spec.tools, *spec.touches))))
        commands.append(
            RegistryCommand(
                group=group,
                name=spec.name,
                owner=spec.owner,
                help_text=spec.help_text,
                tags=tags,
                aliases=tuple(sorted(spec.aliases)),
                internal=spec.internal,
                stable=spec.stable,
            )
        )
        cap = capabilities_for_command(spec.name)
        if cap is None:
            continue
        capabilities.append(
            CapabilityModel(
                subject_kind="command",
                subject_name=spec.name,
                effects=(str(cap.effect_level),),
                tools=tuple(sorted(cap.tools)),
                write_roots=tuple(sorted(cap.writes_allowed_roots)),
                network=bool(cap.network_allowed),
            )
        )
    commands.sort(key=lambda c: (c.group, c.name))

    for chk in checks:
        capabilities.append(
            CapabilityModel(
                subject_kind="check",
                subject_name=chk.check_id,
                effects=tuple(sorted(chk.effects)),
                tools=(),
                write_roots=("artifacts/evidence/",),
                network=("network" in chk.tags),
            )
        )
    capabilities.sort(key=lambda c: (c.subject_kind, c.subject_name))

    defaults, rules, exceptions = load_budgets(root)
    budgets = BudgetModel(defaults={str(k): int(v) for k, v in sorted(defaults.items())}, rules_count=len(rules), exceptions_count=len(exceptions))

    return Registry(
        version=1,
        commands=tuple(commands),
        checks=tuple(checks),
        suites=tuple(sorted(suites, key=lambda s: s.name)),
        owners=tuple(sorted(owner_catalog.owners)),
        budgets=budgets,
        capabilities=tuple(capabilities),
    )


def generate_registry_spine(repo_root: Path | None = None) -> dict[str, Any]:
    root = repo_root or _repo_root()
    payload = load_registry(root).as_generated_payload()
    if jsonschema is not None and (root / REGISTRY_SPINE_SCHEMA).exists():
        schema = json.loads((root / REGISTRY_SPINE_SCHEMA).read_text(encoding="utf-8"))
        jsonschema.validate(payload, schema)
    return payload
