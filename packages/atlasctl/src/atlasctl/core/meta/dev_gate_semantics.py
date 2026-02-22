from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class GateSemantics:
    gate: str
    fast: str
    full: str


DEV_GATE_SEMANTICS: tuple[GateSemantics, ...] = (
    GateSemantics("fmt", "fmt-only", "fmt + atlasctl check run repo"),
    GateSemantics("lint", "lint lane only", "lint lane + atlasctl check run repo"),
    GateSemantics("test", "test lane only", "test lane + atlasctl check run repo"),
    GateSemantics("check", "cargo check only", "cargo check + atlasctl check run repo"),
    GateSemantics("audit", "cargo deny only", "cargo deny + atlasctl check run repo"),
    GateSemantics("docs", "docs lane only", "docs lane + atlasctl check run repo"),
    GateSemantics("ops", "ops lane only", "ops lane + atlasctl check run repo"),
)


def should_run_repo_checks(*, all_variant: bool, and_checks: bool) -> bool:
    return all_variant or and_checks

