from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class OpsSchemaContract:
    id: str
    path: str


def ops_schema_contracts(repo_root: Path) -> tuple[OpsSchemaContract, ...]:
    payload = json.loads((repo_root / "configs/ops/schema-contracts.json").read_text(encoding="utf-8"))
    rows = payload.get("items", [])
    out: list[OpsSchemaContract] = []
    for row in rows:
        out.append(OpsSchemaContract(id=str(row["id"]), path=str(row["path"])))
    return tuple(out)
