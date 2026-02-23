from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def test_dataset_qc_summary_json_contract_and_sample() -> None:
    sample = {
        "schema_version": 1,
        "counts": {"genes": 5, "transcripts": 9, "exons": 11, "cds": 7},
        "orphan_counts": {"transcripts": 1},
        "duplicate_id_events": {"duplicate_gene_ids": 2},
        "contig_stats": {"unknown_contig_feature_ratio": 0.2},
        "rejected_record_count_by_reason": {"bad-contig": 3},
        "biotype_distribution_top_n": [["protein_coding", 4], ["lncRNA", 1]],
    }
    with tempfile.TemporaryDirectory() as td:
        qc = Path(td) / "qc.json"
        out = Path(td) / "qc-summary.json"
        qc.write_text(json.dumps(sample), encoding="utf-8")
        proc = subprocess.run(
            [
                sys.executable,
                "packages/atlasctl/src/atlasctl/commands/ops/datasets/qc_summary.py",
                "--qc",
                str(qc),
                "--out",
                str(out),
                "--format",
                "json",
            ],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        assert proc.returncode == 0, proc.stderr
        payload = json.loads(out.read_text(encoding="utf-8"))
        assert payload["kind"] == "dataset-qc-summary"
        assert payload["counts"]["genes"] == 5

    schema = json.loads((ROOT / "ops/_schemas/datasets/qc-summary.schema.json").read_text(encoding="utf-8"))
    golden = json.loads((ROOT / "ops/datasets/tests/goldens/qc-summary.sample.json").read_text(encoding="utf-8"))
    for key in schema["required"]:
        assert key in golden
    assert golden["kind"] == "dataset-qc-summary"

