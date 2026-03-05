"""Retry policy helpers."""

from dataclasses import dataclass


@dataclass(frozen=True)
class RetryPolicy:
    """Backoff policy for idempotent request retries."""

    attempts: int = 3
    backoff_seconds: float = 0.5
