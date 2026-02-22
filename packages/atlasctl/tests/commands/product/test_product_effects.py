from __future__ import annotations

import json

from atlasctl.core.context import RunContext
from atlasctl.core.effects.product import ProductStep, run_product_lane


def test_run_product_lane_writes_stable_json_report(tmp_path, monkeypatch, capsys) -> None:
    evidence_root = tmp_path / "evidence"
    evidence_root.mkdir(parents=True, exist_ok=True)
    ctx = RunContext.from_args("product-effects", str(evidence_root), "test", False)

    class DummyProc:
        def __init__(self, code: int, stdout: str = "", stderr: str = "") -> None:
            self.returncode = code
            self.stdout = stdout
            self.stderr = stderr

    def fake_run(cmd, cwd=None, env=None, text=True, capture_output=True, timeout_seconds=None):  # type: ignore[no-untyped-def]
        return DummyProc(0, stdout="ok\n")

    monkeypatch.setattr("atlasctl.core.effects.product.process_run", fake_run)

    rc = run_product_lane(ctx, lane="docker build", steps=[ProductStep("step1", ["echo", "ok"])], meta={"x": 1})
    assert rc == 0
    out = capsys.readouterr().out
    assert "product docker build: pass" in out

    report = evidence_root / "product" / "docker-build" / "product-effects" / "report.json"
    assert report.exists()
    payload = json.loads(report.read_text(encoding="utf-8"))
    assert payload["kind"] == "product-lane-report"
    assert payload["lane"] == "docker build"
    assert payload["run_id"] == "product-effects"
    assert payload["summary"]["passed"] == 1
    assert payload["allowed_write_roots"]
    assert payload["lane_contract"]["inputs"]
    assert payload["lane_contract"]["external_tools"]
