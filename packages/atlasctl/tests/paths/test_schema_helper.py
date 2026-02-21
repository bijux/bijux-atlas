from __future__ import annotations

import json
from pathlib import Path

import pytest
from atlasctl.core.contracts.schema import validate_json_file_against_schema
from atlasctl.errors import ScriptError


def test_validate_json_file_against_schema_pass(tmp_path: Path) -> None:
    schema = tmp_path / "schema.json"
    payload = tmp_path / "payload.json"
    schema.write_text(json.dumps({"type": "object", "required": ["x"], "properties": {"x": {"type": "integer"}}}), encoding="utf-8")
    payload.write_text(json.dumps({"x": 1}), encoding="utf-8")
    validate_json_file_against_schema(schema, payload)


def test_validate_json_file_against_schema_fail(tmp_path: Path) -> None:
    schema = tmp_path / "schema.json"
    payload = tmp_path / "payload.json"
    schema.write_text(json.dumps({"type": "object", "required": ["x"], "properties": {"x": {"type": "integer"}}}), encoding="utf-8")
    payload.write_text(json.dumps({"x": "bad"}), encoding="utf-8")
    with pytest.raises(ScriptError):
        validate_json_file_against_schema(schema, payload)
