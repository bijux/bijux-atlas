#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--qc", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    qc = json.loads(Path(args.qc).read_text(encoding="utf-8"))
    counts = qc.get("counts", {})
    dup = qc.get("duplicate_id_events", {})
    contig = qc.get("contig_stats", {})
    rej = qc.get("rejected_record_count_by_reason", {})
    top = qc.get("biotype_distribution_top_n", [])
    lines = [
        "# QC Summary",
        "",
        f"- schema_version: `{qc.get('schema_version', 'unknown')}`",
        f"- genes: `{counts.get('genes', 0)}`",
        f"- transcripts: `{counts.get('transcripts', 0)}`",
        f"- exons: `{counts.get('exons', 0)}`",
        f"- cds: `{counts.get('cds', 0)}`",
        f"- orphan_transcripts: `{qc.get('orphan_counts', {}).get('transcripts', 0)}`",
        f"- duplicate_gene_ids: `{dup.get('duplicate_gene_ids', 0)}`",
        f"- unknown_contig_feature_ratio: `{contig.get('unknown_contig_feature_ratio', 0)}`",
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
            if isinstance(pair, list) and len(pair) == 2:
                lines.append(f"- `{pair[0]}`: `{pair[1]}`")
    else:
        lines.append("- none")
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
