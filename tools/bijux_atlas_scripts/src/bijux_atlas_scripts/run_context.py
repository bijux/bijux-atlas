from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class RunContext:
    run_id: str
    evidence_root: Path
    profile: str
    no_network: bool

    @classmethod
    def from_args(
        cls, run_id: str | None, evidence_root: str | None, profile: str | None, no_network: bool
    ) -> "RunContext":
        resolved_run_id = run_id or os.environ.get("RUN_ID", "local")
        root = Path(evidence_root or os.environ.get("EVIDENCE_ROOT", "artifacts/evidence")).resolve()
        resolved_profile = profile or os.environ.get("PROFILE", "local")
        return cls(run_id=resolved_run_id, evidence_root=root, profile=resolved_profile, no_network=no_network)
