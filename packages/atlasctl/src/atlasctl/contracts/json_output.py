from __future__ import annotations

import json
from pathlib import Path

from ..core.schema import validate_json_file_against_schema
from ..errors import ScriptError
from ..exit_codes import ERR_VALIDATION
from .validate import validate_file


def validate_json_output(schema_name: str, file_path: str, as_json: bool = False) -> int:
    try:
        schema_path = Path(schema_name)
        if schema_path.exists():
            validate_json_file_against_schema(schema_path, Path(file_path))
        else:
            validate_file(schema_name, file_path)
    except Exception as exc:  # pragma: no cover
        raise ScriptError(f"json output validation failed: {exc}", ERR_VALIDATION)
    if as_json:
        print(json.dumps({"status": "ok", "schema": schema_name, "file": file_path}, sort_keys=True))
    return 0
