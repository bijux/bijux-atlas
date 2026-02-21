from __future__ import annotations

from dataclasses import dataclass

from ..checks.registry import check_tags, list_checks


@dataclass(frozen=True)
class SuiteManifest:
    name: str
    check_ids: tuple[str, ...]
    required_env: tuple[str, ...]
    default_effects: tuple[str, ...]
    time_budget_ms: int


_SUITE_DEFS: tuple[tuple[str, str], ...] = (
    ("docs", "docs"),
    ("dev", "dev"),
    ("ops", "ops"),
    ("policies", "policies"),
    ("configs", "configs"),
)

_REQUIRED_ENV: dict[str, tuple[str, ...]] = {
    "docs": ("PYTHONPATH",),
    "dev": ("PYTHONPATH",),
    "ops": ("PYTHONPATH",),
    "policies": ("PYTHONPATH",),
    "configs": ("PYTHONPATH",),
    "all": ("PYTHONPATH",),
}

_DEFAULT_EFFECTS: dict[str, tuple[str, ...]] = {
    "docs": ("read",),
    "dev": ("read", "process"),
    "ops": ("read", "process", "write"),
    "policies": ("read", "process"),
    "configs": ("read",),
    "all": ("read", "process", "write"),
}

_TIME_BUDGET_MS: dict[str, int] = {
    "docs": 120_000,
    "dev": 180_000,
    "ops": 240_000,
    "policies": 240_000,
    "configs": 120_000,
    "all": 480_000,
}


def _checks_for_tag(tag: str) -> tuple[str, ...]:
    ids = [check.check_id for check in list_checks() if tag in check_tags(check)]
    return tuple(sorted(ids))


def load_first_class_suites() -> dict[str, SuiteManifest]:
    suites: dict[str, SuiteManifest] = {}
    all_ids: set[str] = set()
    for name, tag in _SUITE_DEFS:
        check_ids = _checks_for_tag(tag)
        all_ids.update(check_ids)
        suites[name] = SuiteManifest(
            name=name,
            check_ids=check_ids,
            required_env=_REQUIRED_ENV[name],
            default_effects=_DEFAULT_EFFECTS[name],
            time_budget_ms=_TIME_BUDGET_MS[name],
        )
    suites["all"] = SuiteManifest(
        name="all",
        check_ids=tuple(sorted(all_ids)),
        required_env=_REQUIRED_ENV["all"],
        default_effects=_DEFAULT_EFFECTS["all"],
        time_budget_ms=_TIME_BUDGET_MS["all"],
    )
    return suites
