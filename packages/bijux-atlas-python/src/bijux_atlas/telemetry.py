"""Logging and tracing integration points."""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Callable

TraceHook = Callable[[str, dict[str, object]], None]


@dataclass(slots=True)
class Telemetry:
    """Carries optional instrumentation hooks."""

    logger: logging.Logger | None = None
    trace_hook: TraceHook | None = None

    def emit_log(self, message: str, **fields: object) -> None:
        if self.logger is not None:
            self.logger.info("%s | %s", message, fields)

    def emit_trace(self, event: str, **fields: object) -> None:
        if self.trace_hook is not None:
            self.trace_hook(event, fields)
