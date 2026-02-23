from __future__ import annotations

from dataclasses import dataclass, field
from typing import Final, Literal

OpsArea = Literal["stack", "deploy", "k8s", "obs", "load", "e2e", "datasets", "pins", "reports"]

OPS_AREAS: Final[tuple[OpsArea, ...]] = (
    "stack",
    "deploy",
    "k8s",
    "obs",
    "load",
    "e2e",
    "datasets",
    "pins",
    "reports",
)

OPS_ACTIONS_BY_AREA: Final[dict[OpsArea, tuple[str, ...]]] = {
    "stack": ("check", "verify", "report", "up", "down", "status", "validate"),
    "deploy": ("check", "verify", "report", "plan", "apply", "rollback"),
    "k8s": ("check", "verify", "report", "render", "validate", "diff"),
    "obs": ("check", "verify", "report", "lint", "drill"),
    "load": ("check", "verify", "report", "run", "compare"),
    "e2e": ("check", "verify", "report", "run", "validate-results"),
    "datasets": ("check", "verify", "report", "lock", "qc", "validate"),
    "pins": ("check", "verify", "report"),
    "reports": ("check", "verify", "report"),
}


@dataclass(frozen=True)
class OpsRuntimeRequest:
    area: OpsArea
    action: str
    report_format: Literal["text", "json"] = "text"
    args: tuple[str, ...] = ()
    env: dict[str, str] = field(default_factory=dict)
    metadata: dict[str, object] = field(default_factory=dict)


@dataclass(frozen=True)
class OpsRuntimeResponse:
    status_code: int
    status: Literal["pass", "fail"]
    message: str = ""
    report_path: str | None = None
    payload: dict[str, object] | None = None
