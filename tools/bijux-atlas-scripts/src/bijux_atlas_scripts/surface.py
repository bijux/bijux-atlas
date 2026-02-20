from __future__ import annotations

import json
from pathlib import Path


def build_surface() -> dict[str, object]:
    root = Path(__file__).resolve().parents[4]
    ownership = json.loads((root / "configs/meta/ownership.json").read_text(encoding="utf-8"))
    commands = [{"command": command, "owner": owner} for command, owner in sorted(ownership["commands"].items())]
    return {
        "schema_version": 1,
        "commands": commands,
        "path_owners": ownership["paths"],
    }


def run_surface(as_json: bool, out_file: str | None) -> int:
    payload = build_surface()
    if out_file:
        out = Path(out_file)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if as_json:
        print(json.dumps(payload, sort_keys=True))
    else:
        print("Scripts command surface")
        for row in payload["commands"]:
            print(f"- {row['command']}: {row['owner']}")
    return 0
