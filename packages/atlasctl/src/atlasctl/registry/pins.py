from __future__ import annotations

from pathlib import Path

OPS_PIN_REGISTRY_KEYS = ("ops_tools", "ops_stack_versions")


def default_pins_path(repo_root: Path) -> Path:
    return repo_root / "configs" / "ops" / "pins.json"


def ops_pin_registry_keys() -> tuple[str, ...]:
    return OPS_PIN_REGISTRY_KEYS
