from __future__ import annotations

import json
from pathlib import Path
from types import SimpleNamespace

from atlasctl.commands.product.command import _build_manifest_payload
from tests.helpers import run_atlasctl


def _ctx(repo_root: Path) -> object:
    return SimpleNamespace(
        repo_root=repo_root,
        run_id="test-run",
        output_format="json",
        evidence_root=repo_root / "artifacts" / "evidence",
        scripts_artifact_root=repo_root / "artifacts" / "atlasctl" / "run" / "test-run",
        no_network=True,
    )


def _prepare_fake_repo(tmp_path: Path) -> Path:
    repo = tmp_path / "repo"
    (repo / "configs" / "product").mkdir(parents=True)
    schema_src = Path(__file__).resolve().parents[5] / "configs" / "product" / "artifact-manifest.schema.json"
    (repo / "configs" / "product" / "artifact-manifest.schema.json").write_text(schema_src.read_text(encoding="utf-8"), encoding="utf-8")
    (repo / "artifacts" / "chart").mkdir(parents=True)
    (repo / "artifacts" / "chart" / "bijux-atlas-0.0.1.tgz").write_bytes(b"fake-chart-package")
    return repo


def test_product_manifest_schema_validation(tmp_path: Path, monkeypatch) -> None:
    repo = _prepare_fake_repo(tmp_path)
    monkeypatch.setenv("DOCKER_IMAGE", "bijux-atlas:test")
    monkeypatch.setenv("IMAGE_VERSION", "0.0.1-test")
    payload = _build_manifest_payload(_ctx(repo))  # type: ignore[arg-type]
    assert payload["kind"] == "product-artifact-manifest"
    assert payload["schema_version"] == 1
    assert isinstance(payload["artifacts"], list)
    assert payload["artifacts"]


def test_product_manifest_determinism_golden(tmp_path: Path, monkeypatch) -> None:
    # schema-validate-exempt: this golden is a synthetic manifest fixture; schema validation is covered above.
    repo = _prepare_fake_repo(tmp_path)
    monkeypatch.setenv("DOCKER_IMAGE", "bijux-atlas:test")
    monkeypatch.setenv("IMAGE_VERSION", "0.0.1-test")
    payload1 = _build_manifest_payload(_ctx(repo))  # type: ignore[arg-type]
    payload2 = _build_manifest_payload(_ctx(repo))  # type: ignore[arg-type]
    assert payload1 == payload2
    golden = Path(__file__).resolve().parents[2] / "goldens" / "product" / "product-artifact-manifest.synthetic.json.golden"
    assert json.dumps(payload1, indent=2, sort_keys=True) + "\n" == golden.read_text(encoding="utf-8")


def test_product_inventory_output_schema(tmp_path: Path) -> None:
    manifest = tmp_path / "manifest.json"
    manifest.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "kind": "product-artifact-manifest",
                "run_id": "r1",
                "version": "v1",
                "artifacts": [
                    {"id": "docker-image-tag", "path": "bijux-atlas:v1", "kind": "docker-image-tag", "sha256": "0" * 64, "size_bytes": 0}
                ],
            },
            sort_keys=True,
        ),
        encoding="utf-8",
    )
    proc = run_atlasctl("--quiet", "--format", "json", "product", "inventory", "--manifest", str(manifest))
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["kind"] == "product-artifact-manifest"
    assert isinstance(payload["artifacts"], list)
