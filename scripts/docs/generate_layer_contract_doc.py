#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INP = ROOT / "ops" / "_meta" / "layer-contract.json"
OUT = ROOT / "docs" / "_generated" / "layer-contract.md"


def main() -> int:
    c = json.loads(INP.read_text(encoding="utf-8"))
    lines = [
        "# Layer Contract",
        "",
        f"- Contract version: `{c['contract_version']}`",
        f"- Compatibility policy: {c['compatibility']['policy']}",
        "",
        "## Namespaces",
    ]
    for k, v in c["namespaces"].items():
        lines.append(f"- `{k}`: `{v}`")

    lines.extend(["", "## Services"]) 
    for k, v in c["services"].items():
        lines.append(f"- `{k}`: service `{v['service_name']}`, selector `{json.dumps(v['selector'], sort_keys=True)}`")

    lines.extend(["", "## Ports"]) 
    for k, v in c["ports"].items():
        lines.append(f"- `{k}`: `{json.dumps(v, sort_keys=True)}`")

    lines.extend(["", "## Labels", "- Required labels:"])
    for item in c["labels"]["required"]:
        lines.append(f"- `{item}`")

    lines.extend(["", "## Release Metadata"])
    lines.append(f"- Required fields: `{', '.join(c['release_metadata']['required_fields'])}`")
    lines.append(f"- Defaults: `{json.dumps(c['release_metadata']['defaults'], sort_keys=True)}`")

    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
