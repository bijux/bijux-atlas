"""Retry policy helpers."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(slots=True)
class RetryPolicy:
    """Simple linear retry policy."""

    max_retries: int
    backoff_seconds: float
    max_backoff_seconds: float

    def should_retry(
        self,
        attempt_index: int,
        status_code: int | None,
        error: Exception | None,
        *,
        idempotent: bool,
    ) -> bool:
        if not idempotent:
            return False
        if attempt_index >= self.max_retries:
            return False
        if error is not None:
            return True
        if status_code is None:
            return False
        return status_code >= 500 or status_code == 429

    def backoff_delay(self, attempt_index: int) -> float:
        raw = self.backoff_seconds * float(attempt_index + 1)
        return min(raw, self.max_backoff_seconds)
