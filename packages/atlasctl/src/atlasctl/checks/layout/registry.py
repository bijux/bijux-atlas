from __future__ import annotations

from dataclasses import dataclass
from importlib import import_module
from types import ModuleType
from typing import Callable

from pathlib import Path


@dataclass(frozen=True)
class LayoutCheckSpec:
    module: str
    check_id: str
    description: str
    run: Callable[[Path], tuple[int, list[str]]]


# Canonical registry for first-class layout checks.
_LAYOUT_CHECK_MODULES = (
    "atlasctl.checks.layout.root.check_root_shape",
    "atlasctl.checks.layout.root.check_forbidden_paths",
    "atlasctl.checks.layout.root.check_no_direct_script_runs",
    "atlasctl.checks.layout.root.check_root_determinism",
)


def _load_check_module(module_name: str) -> ModuleType:
    module = import_module(module_name)
    for attr in ("CHECK_ID", "DESCRIPTION", "run"):
        if not hasattr(module, attr):
            raise ValueError(f"layout check module {module_name} missing required attribute {attr}")
    return module


def list_layout_checks() -> tuple[LayoutCheckSpec, ...]:
    checks: list[LayoutCheckSpec] = []
    seen: set[str] = set()
    duplicates: set[str] = set()
    for module_name in _LAYOUT_CHECK_MODULES:
        module = _load_check_module(module_name)
        check_id = str(getattr(module, "CHECK_ID"))
        description = str(getattr(module, "DESCRIPTION"))
        run_func = getattr(module, "run")
        if check_id in seen:
            duplicates.add(check_id)
        seen.add(check_id)
        checks.append(LayoutCheckSpec(module_name, check_id, description, run_func))
    if duplicates:
        dup_text = ", ".join(sorted(duplicates))
        raise ValueError(f"duplicate layout check ids: {dup_text}")
    return tuple(sorted(checks, key=lambda row: row.check_id))
