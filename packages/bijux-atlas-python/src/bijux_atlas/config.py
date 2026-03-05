"""SDK configuration models."""

from dataclasses import dataclass


@dataclass(frozen=True)
class ClientConfig:
    """Connection configuration for the SDK client."""

    base_url: str
    timeout_seconds: float = 30.0
