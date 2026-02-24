from __future__ import annotations

from pathlib import Path

from atlasctl.checks.models import CheckResult, EvidenceRef, Timing, Violation, to_json_text


def test_models_roundtrip_golden() -> None:
    result = CheckResult(
        id="checks_ops_surface_contract",
        status="fail",
        violations=(
            Violation(
                code="OPS_SURFACE",
                message="surface mismatch",
                hint="run atlasctl check run --domain ops",
                path="ops/inventory/surfaces.json",
                line=7,
            ),
        ),
        evidence=(
            EvidenceRef(
                kind="json",
                path="artifacts/evidence/run-1/surface.json",
                description="surface snapshot",
                content_type="application/json",
            ),
        ),
        timing=Timing(duration_ms=17, budget_ms=250),
    )
    expected = Path("packages/atlasctl/tests/goldens/check/models-roundtrip.json.golden").read_text(encoding="utf-8")
    assert to_json_text(result) == expected
