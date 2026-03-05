"""Error model for atlas-client."""

from __future__ import annotations


class AtlasClientError(Exception):
    """Base error for all atlas-client failures."""


class AtlasConfigError(AtlasClientError):
    """Raised when a client configuration is invalid."""


class AtlasApiError(AtlasClientError):
    """Raised when the server returns a non-success status."""

    def __init__(self, status_code: int, body: str) -> None:
        super().__init__(f"atlas api error: status={status_code} body={body}")
        self.status_code = status_code
        self.body = body


class AtlasRetryExhaustedError(AtlasClientError):
    """Raised when all retry attempts are exhausted."""

    def __init__(self, attempts: int, last_error: Exception | None = None) -> None:
        details = f" after {attempts} attempts"
        if last_error is not None:
            details = f"{details}: {last_error}"
        super().__init__(f"atlas retry exhausted{details}")
        self.attempts = attempts
        self.last_error = last_error
