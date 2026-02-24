from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .catalogs import load_suites_catalog
from ..checks.registry import check_tags, list_checks


@dataclass(frozen=True)
class SuiteRecord:
    name: str
    includes: tuple[str, ...]
    item_count: int
    complete: bool
    tags: tuple[str, ...]
    internal: bool = False


@dataclass(frozen=True)
class SuiteManifestSpec:
    name: str
    markers: tuple[str, ...]
    required_env: tuple[str, ...]
    default_effects: tuple[str, ...]
    time_budget_ms: int
    include_checks: tuple[str, ...] = ()
    exclude_markers: tuple[str, ...] = ()
    internal: bool = False


_SUITES_CATALOG = Path(__file__).resolve().with_name("suites_catalog.json")

_SUITE_MARKERS: tuple[str, ...] = ("required", "ci", "local", "slow")
OPS_SUITE_NAMES: tuple[str, ...] = (
    "ops",
    "ops-stack",
    "ops-deploy",
    "ops-load",
    "ops-obs",
)


def suite_manifest_specs() -> tuple[SuiteManifestSpec, ...]:
    payload = load_suites_catalog(_SUITES_CATALOG.parents[5])
    rows = payload.get("suites", [])
    specs: list[SuiteManifestSpec] = []
    for row in rows:
        raw_markers = tuple(str(x) for x in row.get("markers", []))
        normalized_markers = tuple(sorted(set(raw_markers)))
        specs.append(
            SuiteManifestSpec(
                name=str(row["name"]),
                markers=normalized_markers,
                required_env=tuple(str(x) for x in row.get("required_env", [])),
                default_effects=tuple(str(x) for x in row.get("default_effects", [])),
                time_budget_ms=int(row.get("time_budget_ms", 0)),
                include_checks=tuple(str(x) for x in row.get("include_checks", [])),
                exclude_markers=tuple(str(x) for x in row.get("exclude_markers", [])),
                internal=bool(row.get("internal", False)),
            )
        )
    return tuple(sorted(specs, key=lambda spec: spec.name))


def suite_marker_violations() -> list[str]:
    payload = load_suites_catalog(_SUITES_CATALOG.parents[5])
    rows = payload.get("suites", [])
    errors: list[str] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        name = str(row.get("name", ""))
        markers = tuple(str(x) for x in row.get("markers", []))
        if markers != tuple(sorted(set(markers))):
            errors.append(f"suite `{name}` markers must be sorted and unique")
        unknown = set(markers).difference(_SUITE_MARKERS)
        if unknown:
            errors.append(f"suite `{name}` has unknown markers: {sorted(unknown)}")
    return sorted(errors)


def suite_markers() -> tuple[str, ...]:
    return _SUITE_MARKERS


def ops_suite_names() -> tuple[str, ...]:
    return OPS_SUITE_NAMES


def resolve_check_ids(spec: SuiteManifestSpec) -> tuple[str, ...]:
    out: set[str] = set(spec.include_checks)
    marker_set = set(spec.markers)
    excluded = set(spec.exclude_markers)
    for check in list_checks():
        tags = set(check_tags(check))
        if marker_set and marker_set.isdisjoint(tags):
            continue
        if excluded.intersection(tags):
            continue
        out.add(check.check_id)
    return tuple(sorted(out))
