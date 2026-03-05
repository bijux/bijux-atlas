from __future__ import annotations

import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]


class TutorialDatasetContractTests(unittest.TestCase):
    def test_metadata_contract_fields(self) -> None:
        contract = json.loads((ROOT / "tutorials/contracts/tutorial-dataset-contract.json").read_text())
        required = set(contract["required"])

        for dataset in [
            "atlas-example-minimal",
            "atlas-example-medium",
            "atlas-example-large-synthetic",
        ]:
            metadata_path = ROOT / f"configs/examples/datasets/{dataset}/metadata.json"
            metadata = json.loads(metadata_path.read_text())
            self.assertTrue(required.issubset(metadata.keys()))
            self.assertGreater(int(metadata["record_count"]), 0)

    def test_gene_row_required_fields(self) -> None:
        row = (
            ROOT
            / "configs/examples/datasets/atlas-example-minimal/genes.jsonl"
        ).read_text(encoding="utf-8").splitlines()[0]
        data = json.loads(row)
        for field in ["gene_id", "symbol", "chromosome", "biotype", "length_bp"]:
            self.assertIn(field, data)


if __name__ == "__main__":
    unittest.main()
