#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACT_DIR = ROOT / "docs" / "contracts"

for path in sorted(CONTRACT_DIR.glob("*.json")):
    data = json.loads(path.read_text())
    if isinstance(data, dict):
        if "codes" in data and isinstance(data["codes"], list):
            data["codes"] = sorted(data["codes"])
        if "top_level_keys" in data and isinstance(data["top_level_keys"], list):
            data["top_level_keys"] = sorted(data["top_level_keys"])
        if "metrics" in data and isinstance(data["metrics"], list):
            data["metrics"] = sorted(
                [
                    {
                        "name": m["name"],
                        "labels": sorted(m.get("labels", [])),
                    }
                    for m in data["metrics"]
                ],
                key=lambda x: x["name"],
            )
        if "spans" in data and isinstance(data["spans"], list):
            data["spans"] = sorted(
                [
                    {
                        "name": s["name"],
                        "required_attributes": sorted(s.get("required_attributes", [])),
                    }
                    for s in data["spans"]
                ],
                key=lambda x: x["name"],
            )
        if "env_keys" in data and isinstance(data["env_keys"], list):
            data["env_keys"] = sorted(data["env_keys"])
        if "endpoints" in data and isinstance(data["endpoints"], list):
            data["endpoints"] = sorted(
                data["endpoints"], key=lambda e: (e["path"], e["method"])
            )

    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")

print("contracts formatted")
