from __future__ import annotations

import json
from pathlib import Path

from ..core.schema import validate_json_file_against_schema
from ..errors import ScriptError
from ..exit_codes import ERR_VALIDATION


def validate_json_output(schema_path: str, file_path: str, as_json: bool = False) -> int:
    try:
        validate_json_file_against_schema(Path(schema_path), Path(file_path))
    except Exception as exc:  # pragma: no cover
        raise ScriptError(f"json output validation failed: {exc}", ERR_VALIDATION)
    if as_json:
        print(json.dumps({"status": "ok", "schema": schema_path, "file": file_path}, sort_keys=True))
    return 0
