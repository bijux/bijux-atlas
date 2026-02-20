from __future__ import annotations

from pathlib import Path


def default_pins_path(repo_root: Path) -> Path:
    return repo_root / "configs" / "ops" / "pins.json"
