from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class GateSemantics:
    gate: str
    fast: str
    full: str


DEV_GATE_SEMANTICS: tuple[GateSemantics, ...] = (
    GateSemantics("fmt", "fmt-only", "fmt full variant"),
    GateSemantics("lint", "lint lane only", "lint full variant"),
    GateSemantics("test", "test lane only", "test full variant"),
    GateSemantics("check", "cargo check only", "cargo check full variant"),
    GateSemantics("audit", "cargo deny only", "cargo deny full variant"),
    GateSemantics("docs", "docs lane only", "docs full variant"),
    GateSemantics("ops", "ops lane only", "ops full variant"),
)


def should_run_repo_checks(*, all_variant: bool, and_checks: bool) -> bool:
    _ = all_variant
    return and_checks
