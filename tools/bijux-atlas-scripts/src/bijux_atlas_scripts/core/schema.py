from __future__ import annotations

import json
from pathlib import Path


def validate_json_file_against_schema(schema_path: Path, payload_path: Path) -> None:
    import jsonschema

    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    payload = json.loads(payload_path.read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
