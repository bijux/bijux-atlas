"""Configuration schema for bijux-atlas."""

from __future__ import annotations

import os
from dataclasses import dataclass, field

from .errors import AtlasConfigError


@dataclass(slots=True)
class ClientConfig:
    """Runtime configuration for AtlasClient."""

    base_url: str
    timeout_seconds: float = 10.0
    max_retries: int = 2
    backoff_seconds: float = 0.2
    max_backoff_seconds: float = 5.0
    auth_token: str | None = None
    verify_ssl: bool = True
    proxy_url: str | None = None
    user_agent: str = "bijux-atlas/0.1.0"
    default_headers: dict[str, str] = field(default_factory=dict)
    request_id: str | None = None
    validate_response_schema: bool = False

    def validate(self) -> None:
        if not self.base_url.startswith(("http://", "https://")):
            raise AtlasConfigError("base_url must start with http:// or https://")
        if self.timeout_seconds <= 0:
            raise AtlasConfigError("timeout_seconds must be > 0")
        if self.max_retries < 0:
            raise AtlasConfigError("max_retries must be >= 0")
        if self.backoff_seconds < 0:
            raise AtlasConfigError("backoff_seconds must be >= 0")
        if self.max_backoff_seconds < 0:
            raise AtlasConfigError("max_backoff_seconds must be >= 0")
        if self.max_backoff_seconds < self.backoff_seconds:
            raise AtlasConfigError("max_backoff_seconds must be >= backoff_seconds")
        if self.proxy_url is not None and not self.proxy_url.startswith(("http://", "https://")):
            raise AtlasConfigError("proxy_url must start with http:// or https://")

    @classmethod
    def from_env(cls) -> "ClientConfig":
        """Build config from standard BIJUX_ATLAS_* environment variables."""

        base_url = os.getenv("BIJUX_ATLAS_URL")
        if not base_url:
            raise AtlasConfigError("BIJUX_ATLAS_URL is required")

        timeout_seconds = float(os.getenv("BIJUX_ATLAS_TIMEOUT_SECONDS", "10.0"))
        max_retries = int(os.getenv("BIJUX_ATLAS_MAX_RETRIES", "2"))
        backoff_seconds = float(os.getenv("BIJUX_ATLAS_BACKOFF_SECONDS", "0.2"))
        max_backoff_seconds = float(os.getenv("BIJUX_ATLAS_MAX_BACKOFF_SECONDS", "5.0"))
        auth_token = os.getenv("BIJUX_ATLAS_TOKEN")
        proxy_url = os.getenv("BIJUX_ATLAS_PROXY")
        verify_ssl = os.getenv("BIJUX_ATLAS_VERIFY_SSL", "1") not in {"0", "false", "False"}
        request_id = os.getenv("BIJUX_ATLAS_REQUEST_ID")

        return cls(
            base_url=base_url,
            timeout_seconds=timeout_seconds,
            max_retries=max_retries,
            backoff_seconds=backoff_seconds,
            max_backoff_seconds=max_backoff_seconds,
            auth_token=auth_token,
            verify_ssl=verify_ssl,
            proxy_url=proxy_url,
            request_id=request_id,
        )
