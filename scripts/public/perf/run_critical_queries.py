#!/usr/bin/env python3
# owner: platform
# purpose: execute critical SQLite query shapes against fixture DB and enforce explain-plan/index contracts.
# stability: public
# called-by: make query-plan-gate, make critical-query-check
# Purpose: run SSOT critical queries from configs/perf/critical_queries.json on deterministic fixture data.
# Inputs: critical queries json, sqlite indexes contract, output path arguments.
# Outputs: writes deterministic explain snapshot json; exits non-zero on missing index usage or full scan.
from __future__ import annotations

import argparse
import json
import sqlite3
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CRITICAL_QUERIES_PATH = ROOT / "configs" / "perf" / "critical_queries.json"
SQLITE_INDEX_CONTRACT_PATH = ROOT / "docs" / "contracts" / "SQLITE_INDEXES.json"
SQLITE_SCHEMA_PATH = ROOT / "crates" / "bijux-atlas-ingest" / "sql" / "schema_v4.sql"
DEFAULT_OUT = ROOT / "artifacts" / "isolates" / "query-plan-gate" / "critical-query-explain.json"
SNAPSHOT_PATH = ROOT / "configs" / "perf" / "critical_queries_explain_snapshot.json"


def setup_fixture_db(conn: sqlite3.Connection) -> None:
    conn.executescript(SQLITE_SCHEMA_PATH.read_text(encoding="utf-8"))

    rows = [
        (1, "gene1", "BRCA1", "brca1", "protein_coding", "chr1", 10, 40, 2, 31),
        (2, "gene2", "BRCA2", "brca2", "protein_coding", "chr1", 50, 90, 1, 41),
        (3, "gene3", "TP53", "tp53", "lncRNA", "chr2", 5, 25, 1, 21),
        (4, "gene4", "TNF", "tnf", "lncRNA", "chr2", 30, 45, 1, 16),
    ]
    for r in rows:
        conn.execute(
            """
            INSERT INTO gene_summary (
              id, gene_id, name, name_normalized, biotype, seqid, start, [end],
              transcript_count, exon_count, total_exon_span, cds_present, sequence_length
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, ?)
            """,
            (r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9]),
        )
        conn.execute(
            "INSERT INTO gene_summary_rtree (gene_rowid, start, [end]) VALUES (?, ?, ?)",
            (r[0], float(r[6]), float(r[7])),
        )
    conn.commit()


def assert_required_indexes(conn: sqlite3.Connection, index_contract: dict) -> None:
    existing = {
        row[0]
        for row in conn.execute(
            "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'"
        )
    }
    required = set()
    for names in index_contract["required_indexes"].values():
        required.update(names)
    missing = sorted(required - existing)
    if missing:
        raise SystemExit(f"missing required indexes: {', '.join(missing)}")

    existing_virtual = {
        row[0] for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")
    }
    for vt in index_contract.get("required_virtual_tables", []):
        if vt not in existing_virtual:
            raise SystemExit(f"missing required virtual table: {vt}")


def explain_lines(conn: sqlite3.Connection, sql: str, params: list) -> list[str]:
    rows = conn.execute(f"EXPLAIN QUERY PLAN {sql}", params).fetchall()
    lines = sorted(str(row[3]) for row in rows)
    return lines


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    ap.add_argument("--update-snapshot", action="store_true")
    args = ap.parse_args()

    critical = json.loads(CRITICAL_QUERIES_PATH.read_text(encoding="utf-8"))
    contract = json.loads(SQLITE_INDEX_CONTRACT_PATH.read_text(encoding="utf-8"))

    conn = sqlite3.connect(":memory:")
    setup_fixture_db(conn)
    assert_required_indexes(conn, contract)

    actual: dict[str, list[str]] = {}
    for q in critical["queries"]:
        lines = explain_lines(conn, q["sql"], q["params"])
        plan_joined = " | ".join(lines).lower()

        for needle in q.get("required_plan_substrings", []):
            if needle.lower() not in plan_joined:
                raise SystemExit(
                    f"critical query `{q['id']}` missing required plan substring `{needle}`: {plan_joined}"
                )

        for scan in contract.get("no_full_scan_patterns", []):
            if scan.lower() in plan_joined and "using index" not in plan_joined and "virtual table index" not in plan_joined:
                raise SystemExit(
                    f"critical query `{q['id']}` violated no-full-scan policy: {plan_joined}"
                )

        actual[q["id"]] = lines

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(actual, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if args.update_snapshot:
        SNAPSHOT_PATH.write_text(json.dumps(actual, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"updated snapshot: {SNAPSHOT_PATH}")
        return 0

    if not SNAPSHOT_PATH.exists():
        raise SystemExit(f"snapshot missing: {SNAPSHOT_PATH}; run with --update-snapshot")

    expected = json.loads(SNAPSHOT_PATH.read_text(encoding="utf-8"))
    if expected != actual:
        raise SystemExit(
            "critical query explain snapshot drift detected. "
            f"expected={SNAPSHOT_PATH} actual={args.out}"
        )

    print("critical query gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
