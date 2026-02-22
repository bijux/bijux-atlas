from __future__ import annotations

import json
from pathlib import Path

from ...contracts.ids import SURFACE
from ...contracts.validate_self import validate_self
from ...core.fs import ensure_evidence_path
from ...core.runtime.paths import write_text_file


def build_surface(run_id: str) -> dict[str, object]:
    root = next((parent for parent in Path(__file__).resolve().parents if (parent / ".git").exists()), None)
    if root is None:
        raise RuntimeError("unable to locate repository root")
    ownership = json.loads((root / "configs/meta/ownership.json").read_text(encoding="utf-8"))
    commands = [{"command": command, "owner": owner} for command, owner in sorted(ownership["commands"].items())]
    return {
        "schema_name": SURFACE,
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "ok": True,
        "errors": [],
        "warnings": [],
        "meta": {"source": "configs/meta/ownership.json"},
        "run_id": run_id,
        "commands": commands,
        "path_owners": ownership["paths"],
    }


def run_surface(as_json: bool, out_file: str | None, ctx=None) -> int:
    run_id = ctx.run_id if ctx is not None else "unknown"
    payload = build_surface(run_id=run_id)
    validate_self(SURFACE, payload)
    if out_file:
        out = ensure_evidence_path(ctx, Path(out_file)) if ctx is not None else Path(out_file)
        write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if as_json:
        print(json.dumps(payload, sort_keys=True))
    else:
        print("Scripts command surface")
        for row in payload["commands"]:
            print(f"- {row['command']}: {row['owner']}")
    return 0
