from __future__ import annotations

from ..core.runtime.serialize import dumps_json


def render_error(*, as_json: bool, message: str, code: int, kind: str = "generic_error", run_id: str = "") -> str:
    if as_json:
        return dumps_json(
            {
                "schema_name": "atlasctl.error.v1",
                "schema_version": 1,
                "tool": "atlasctl",
                "run_id": run_id,
                "status": "error",
                "errors": [{"code": code, "kind": kind, "message": message}],
            },
            pretty=False,
        )
    return message
