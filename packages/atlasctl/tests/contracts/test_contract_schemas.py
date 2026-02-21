from __future__ import annotations

import json
from pathlib import Path

import pytest

from atlasctl.contracts.validate import load_catalog, validate


ROOT = Path(__file__).resolve().parents[3]


@pytest.mark.unit
def test_all_catalog_schemas_have_files() -> None:
    catalog = load_catalog()
    assert catalog
    schemas_root = ROOT / "packages/atlasctl/src/atlasctl/contracts/schemas"
    for entry in catalog.values():
        assert (schemas_root / entry.file).is_file(), entry.file


@pytest.mark.unit
def test_sample_payloads_validate_against_catalog() -> None:
    samples = sorted((ROOT / "packages/atlasctl/tests/goldens/samples").glob("*.json"))
    assert samples
    for path in samples:
        payload = json.loads(path.read_text(encoding="utf-8"))
        validate(payload["schema_name"], payload)
