from __future__ import annotations

from pathlib import Path
from typing import Any


def load_yaml(path: Path) -> Any:
    try:
        import yaml  # type: ignore
    except Exception as exc:  # pragma: no cover
        raise RuntimeError('PyYAML is required for YAML parsing') from exc
    with path.open('r', encoding='utf-8') as f:
        return yaml.safe_load(f)


def validate_yaml_required_keys(path: Path, required_keys: list[str]) -> list[str]:
    data = load_yaml(path)
    if not isinstance(data, dict):
        return [f'{path}: root must be mapping']
    missing = [k for k in required_keys if k not in data]
    return [f'{path}: missing key `{k}`' for k in missing]
