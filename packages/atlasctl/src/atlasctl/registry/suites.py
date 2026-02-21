from __future__ import annotations

from dataclasses import dataclass

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


_SUITE_MANIFEST_SPECS: tuple[SuiteManifestSpec, ...] = (
    SuiteManifestSpec("docs", ("docs",), ("PYTHONPATH",), ("read",), 120_000),
    SuiteManifestSpec("dev", ("dev",), ("PYTHONPATH",), ("read", "process"), 180_000),
    SuiteManifestSpec("ops", ("ops",), ("PYTHONPATH",), ("read", "process", "write"), 240_000),
    SuiteManifestSpec("policies", ("policies",), ("PYTHONPATH",), ("read", "process"), 240_000),
    SuiteManifestSpec("configs", ("configs",), ("PYTHONPATH",), ("read",), 120_000),
    SuiteManifestSpec("local", ("fast",), ("PYTHONPATH",), ("read", "process"), 120_000),
    SuiteManifestSpec("slow", ("slow",), ("PYTHONPATH",), ("read", "process"), 300_000),
    SuiteManifestSpec("required", ("required",), ("PYTHONPATH",), ("read", "process"), 300_000),
    SuiteManifestSpec("ci", ("required",), ("PYTHONPATH",), ("read", "process"), 420_000),
    SuiteManifestSpec(
        "required_proof",
        ("required",),
        ("PYTHONPATH",),
        ("read", "process"),
        480_000,
        include_checks=(
            "repo.dir_budget_py_files",
            "repo.single_registry_module",
            "repo.legacy_package_absent",
            "repo.legacy_zero_importers",
        ),
    ),
    SuiteManifestSpec(
        "all",
        ("docs", "dev", "ops", "policies", "configs"),
        ("PYTHONPATH",),
        ("read", "process", "write"),
        480_000,
        exclude_markers=("internal", "internal-only"),
    ),
    SuiteManifestSpec(
        "internal",
        ("internal", "internal-only"),
        ("PYTHONPATH",),
        ("read", "process", "write"),
        600_000,
        internal=True,
    ),
)

_SUITE_MARKERS: tuple[str, ...] = ("required", "ci", "local", "slow")


def suite_manifest_specs() -> tuple[SuiteManifestSpec, ...]:
    return _SUITE_MANIFEST_SPECS


def suite_markers() -> tuple[str, ...]:
    return _SUITE_MARKERS


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
