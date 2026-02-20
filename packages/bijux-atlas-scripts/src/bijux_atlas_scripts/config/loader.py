from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def repo_root() -> Path:
    return Path(__file__).resolve().parents[5]


def load_json_config(rel_path: str) -> dict[str, Any]:
    path = repo_root() / rel_path
    return json.loads(path.read_text(encoding="utf-8"))
