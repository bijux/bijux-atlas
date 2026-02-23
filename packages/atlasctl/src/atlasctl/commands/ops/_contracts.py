from __future__ import annotations

from dataclasses import dataclass, field
from typing import Literal

from atlasctl.ops.registry import OPS_ACTIONS_BY_AREA, OPS_AREAS, OpsArea


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
