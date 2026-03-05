"""Configuration schema for atlas-client."""

from __future__ import annotations

from dataclasses import dataclass, field

from .errors import AtlasConfigError


@dataclass(slots=True)
class ClientConfig:
    """Runtime configuration for AtlasClient."""

    base_url: str
    timeout_seconds: float = 10.0
    max_retries: int = 2
    backoff_seconds: float = 0.2
    user_agent: str = "atlas-client/0.1.0"
    default_headers: dict[str, str] = field(default_factory=dict)

    def validate(self) -> None:
        if not self.base_url.startswith(("http://", "https://")):
            raise AtlasConfigError("base_url must start with http:// or https://")
        if self.timeout_seconds <= 0:
            raise AtlasConfigError("timeout_seconds must be > 0")
        if self.max_retries < 0:
            raise AtlasConfigError("max_retries must be >= 0")
        if self.backoff_seconds < 0:
            raise AtlasConfigError("backoff_seconds must be >= 0")
