#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def _summary_payload(qc: dict[str, object]) -> dict[str, object]:
    counts = qc.get("counts", {}) if isinstance(qc.get("counts"), dict) else {}
    dup = qc.get("duplicate_id_events", {}) if isinstance(qc.get("duplicate_id_events"), dict) else {}
    contig = qc.get("contig_stats", {}) if isinstance(qc.get("contig_stats"), dict) else {}
    rej = qc.get("rejected_record_count_by_reason", {}) if isinstance(qc.get("rejected_record_count_by_reason"), dict) else {}
    top = qc.get("biotype_distribution_top_n", [])
    return {
        "schema_version": 1,
        "kind": "dataset-qc-summary",
        "qc_schema_version": qc.get("schema_version", "unknown"),
        "counts": {
            "genes": int(counts.get("genes", 0)),
            "transcripts": int(counts.get("transcripts", 0)),
            "exons": int(counts.get("exons", 0)),
            "cds": int(counts.get("cds", 0)),
        },
        "orphan_counts": {
            "transcripts": int((qc.get("orphan_counts", {}) or {}).get("transcripts", 0)),
        },
        "duplicate_gene_ids": int(dup.get("duplicate_gene_ids", 0)),
        "unknown_contig_feature_ratio": contig.get("unknown_contig_feature_ratio", 0),
        "rejections_by_reason": {str(k): int(v) for k, v in sorted(rej.items())},
        "top_biotypes": [
            {"name": str(pair[0]), "count": int(pair[1])}
            for pair in top
            if isinstance(pair, list) and len(pair) == 2
        ],
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--qc", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--format", choices=("markdown", "json"), default="markdown")
    args = ap.parse_args()
    qc = json.loads(Path(args.qc).read_text(encoding="utf-8"))
    summary = _summary_payload(qc)
    counts = summary["counts"]
    rej = summary["rejections_by_reason"]
    top = summary["top_biotypes"]
    lines = [
        "# QC Summary",
        "",
        f"- schema_version: `{summary.get('qc_schema_version', 'unknown')}`",
        f"- genes: `{counts.get('genes', 0)}`",
        f"- transcripts: `{counts.get('transcripts', 0)}`",
        f"- exons: `{counts.get('exons', 0)}`",
        f"- cds: `{counts.get('cds', 0)}`",
        f"- orphan_transcripts: `{summary.get('orphan_counts', {}).get('transcripts', 0)}`",
        f"- duplicate_gene_ids: `{summary.get('duplicate_gene_ids', 0)}`",
        f"- unknown_contig_feature_ratio: `{summary.get('unknown_contig_feature_ratio', 0)}`",
        "",
        "## Rejections By Reason",
    ]
    if rej:
        for k in sorted(rej):
            lines.append(f"- `{k}`: `{rej[k]}`")
    else:
        lines.append("- none")
    lines.append("")
    lines.append("## Top Biotypes")
    if top:
        for pair in top:
            lines.append(f"- `{pair['name']}`: `{pair['count']}`")
    else:
        lines.append("- none")
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    if args.format == "json":
        out.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    else:
        out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
