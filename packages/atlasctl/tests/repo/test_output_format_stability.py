from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.contracts.test_guardrails import check_output_format_stability


def test_output_format_stability_check_passes() -> None:
    root = Path(__file__).resolve().parents[4]
    code, errors = check_output_format_stability(root)
    assert code == 0, errors
