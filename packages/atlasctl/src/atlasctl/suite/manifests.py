from __future__ import annotations

from dataclasses import dataclass

from ..registry.suites import resolve_check_ids, suite_manifest_specs


@dataclass(frozen=True)
class SuiteManifest:
    name: str
    markers: tuple[str, ...]
    check_ids: tuple[str, ...]
    required_env: tuple[str, ...]
    default_effects: tuple[str, ...]
    time_budget_ms: int
    internal: bool = False


def load_first_class_suites() -> dict[str, SuiteManifest]:
    suites: dict[str, SuiteManifest] = {}
    for spec in suite_manifest_specs():
        suites[spec.name] = SuiteManifest(
            name=spec.name,
            markers=spec.markers,
            check_ids=resolve_check_ids(spec),
            required_env=spec.required_env,
            default_effects=spec.default_effects,
            time_budget_ms=spec.time_budget_ms,
            internal=spec.internal,
        )
    return suites
