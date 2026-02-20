#!/usr/bin/env python3
# Purpose: validate required SQLite indexes contract against schema SSOT.
# Inputs: docs/contracts/SQLITE_INDEXES.json and crates/bijux-atlas-ingest/sql/schema_v4.sql.
# Outputs: non-zero when required indexes or virtual tables drift from schema SSOT.
from __future__ import annotations

import json
import sqlite3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONTRACT_PATH = ROOT / "docs" / "contracts" / "SQLITE_INDEXES.json"
SCHEMA_PATH = ROOT / "crates" / "bijux-atlas-ingest" / "sql" / "schema_v4.sql"


def main() -> int:
    contract = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))

    conn = sqlite3.connect(":memory:")
    conn.executescript(SCHEMA_PATH.read_text(encoding="utf-8"))

    existing_indexes = {
        row[0]
        for row in conn.execute(
            "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'"
        )
    }
    expected_indexes = set()
    for names in contract["required_indexes"].values():
        expected_indexes.update(names)

    missing_indexes = sorted(expected_indexes - existing_indexes)
    if missing_indexes:
        raise SystemExit(
            "sqlite indexes contract drift: missing required indexes: "
            + ", ".join(missing_indexes)
        )

    existing_tables = {
        row[0] for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")
    }
    missing_vtables = sorted(
        set(contract.get("required_virtual_tables", [])) - existing_tables
    )
    if missing_vtables:
        raise SystemExit(
            "sqlite indexes contract drift: missing required virtual tables: "
            + ", ".join(missing_vtables)
        )

    print("sqlite indexes contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
