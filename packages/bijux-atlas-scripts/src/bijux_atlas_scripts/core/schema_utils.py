from __future__ import annotations

import json
from pathlib import Path


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_json(payload: dict[str, object], schema_path: Path) -> None:
    import jsonschema

    schema = load_json(schema_path)
    jsonschema.validate(payload, schema)
