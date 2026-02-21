from __future__ import annotations

import json
from pathlib import Path

from ..errors import ScriptError
from ..exit_codes import ERR_VALIDATION


def validate_json_file_against_schema(schema_path: Path, payload_path: Path) -> None:
    import jsonschema

    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    payload = json.loads(payload_path.read_text(encoding="utf-8"))
    try:
        jsonschema.validate(payload, schema)
    except jsonschema.ValidationError as exc:
        pointer = "/".join(str(p) for p in exc.absolute_path)
        loc = pointer or "<root>"
        raise ScriptError(f"schema validation failed at {loc}: {exc.message}", ERR_VALIDATION) from exc
