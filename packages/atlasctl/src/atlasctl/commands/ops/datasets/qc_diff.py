#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def get(v, path, default=0):
    cur = v
    for p in path:
        if not isinstance(cur, dict):
            return default
        cur = cur.get(p)
    return cur if cur is not None else default


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--target", required=True)
    args = ap.parse_args()
    base = json.loads(Path(args.base).read_text(encoding="utf-8"))
    target = json.loads(Path(args.target).read_text(encoding="utf-8"))
    keys = [
        ("counts.genes", ["counts", "genes"]),
        ("counts.transcripts", ["counts", "transcripts"]),
        ("counts.exons", ["counts", "exons"]),
        ("counts.cds", ["counts", "cds"]),
        ("orphan_counts.transcripts", ["orphan_counts", "transcripts"]),
        ("duplicate_id_events.duplicate_gene_ids", ["duplicate_id_events", "duplicate_gene_ids"]),
        ("contig_stats.unknown_contig_feature_ratio", ["contig_stats", "unknown_contig_feature_ratio"]),
    ]
    out = {"base": args.base, "target": args.target, "changes": []}
    for name, path in keys:
        out["changes"].append({"field": name, "base": get(base, path), "target": get(target, path)})
    print(json.dumps(out, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
