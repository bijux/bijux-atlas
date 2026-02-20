from __future__ import annotations

import json
from pathlib import Path

from .errors import ScriptError
from .exit_codes import ERR_VALIDATION


def validate_json_output(schema_path: str, file_path: str, as_json: bool = False) -> int:
    try:
        import jsonschema
    except Exception as exc:  # pragma: no cover
        raise ScriptError(f"jsonschema dependency missing: {exc}", ERR_VALIDATION)

    schema = json.loads(Path(schema_path).read_text(encoding="utf-8"))
    payload = json.loads(Path(file_path).read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    if as_json:
        print(json.dumps({"status": "ok", "schema": schema_path, "file": file_path}, sort_keys=True))
    return 0
