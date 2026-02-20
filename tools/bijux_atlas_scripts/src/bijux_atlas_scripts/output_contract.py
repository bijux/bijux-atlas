from __future__ import annotations

import json
from pathlib import Path


def validate_json_output(schema_path: str, file_path: str) -> int:
    try:
        import jsonschema
    except Exception as exc:  # pragma: no cover
        raise SystemExit(f"jsonschema dependency missing: {exc}")

    schema = json.loads(Path(schema_path).read_text(encoding="utf-8"))
    payload = json.loads(Path(file_path).read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    return 0
