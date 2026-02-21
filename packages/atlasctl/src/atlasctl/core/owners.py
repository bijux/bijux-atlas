from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path


OWNERS_JSON = Path("configs/meta/owners.json")


@dataclass(frozen=True)
class OwnerCatalog:
    owners: tuple[str, ...]
    command_groups: dict[str, str]


def load_owner_catalog(repo_root: Path) -> OwnerCatalog:
    payload = json.loads((repo_root / OWNERS_JSON).read_text(encoding="utf-8"))
    rows = payload.get("owners", [])
    owners = tuple(sorted({str(row.get("id", "")).strip() for row in rows if str(row.get("id", "")).strip()}))
    command_groups = {str(k): str(v) for k, v in dict(payload.get("command_groups", {})).items()}
    return OwnerCatalog(owners=owners, command_groups=command_groups)

