from __future__ import annotations

from typing import Any

from ..ids import OUTPUT_BASE_V1, OUTPUT_BASE_V2
from ..schema.validate_self import validate_self


def build_output_base(
    *,
    run_id: str,
    ok: bool,
    errors: list[str] | None = None,
    warnings: list[str] | None = None,
    meta: dict[str, Any] | None = None,
    version: int = 1,
) -> dict[str, Any]:
    if version == 2:
        payload: dict[str, Any] = {
            "schema_name": OUTPUT_BASE_V2,
            "schema_version": 2,
            "tool": "atlasctl",
            "ok": ok,
            "errors": sorted(errors or []),
            "warnings": sorted(warnings or []),
            "meta": meta or {},
            "run_id": run_id,
            "contract_version": 2,
        }
        return validate_self(OUTPUT_BASE_V2, payload)
    payload = {
        "schema_name": OUTPUT_BASE_V1,
        "schema_version": 1,
        "tool": "atlasctl",
        "ok": ok,
        "errors": sorted(errors or []),
        "warnings": sorted(warnings or []),
        "meta": meta or {},
        "run_id": run_id,
    }
    return validate_self(OUTPUT_BASE_V1, payload)
