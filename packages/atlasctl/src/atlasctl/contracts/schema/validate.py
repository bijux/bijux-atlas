from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from ...errors import ScriptError
from ...exit_codes import ERR_VALIDATION
from .catalog import load_catalog, schema_path_for


def validate(schema_name: str, payload: Any) -> None:
    import jsonschema

    schema = json.loads(schema_path_for(schema_name).read_text(encoding="utf-8"))
    try:
        jsonschema.validate(payload, schema)
    except jsonschema.ValidationError as exc:
        pointer = "/".join(str(p) for p in exc.absolute_path)
        loc = pointer or "<root>"
        raise ScriptError(f"schema validation failed for {schema_name} at {loc}: {exc.message}", ERR_VALIDATION) from exc


def validate_file(schema_name: str, file_path: str | Path) -> None:
    payload = json.loads(Path(file_path).read_text(encoding="utf-8"))
    validate(schema_name, payload)
