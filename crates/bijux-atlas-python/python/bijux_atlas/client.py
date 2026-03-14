"""Top-level Atlas client."""

from __future__ import annotations

import json
import logging
import pathlib
import warnings
from collections.abc import Iterator

from .config import ClientConfig
from .http import HttpTransport
from .pagination import Page, next_page_token, page_items
from .query import QueryRequest
from .retry import RetryPolicy
from .telemetry import Telemetry, TraceHook
from .version import __version__


class AtlasClient:
    """Client for Atlas query API endpoints."""

    def __init__(
        self,
        config: ClientConfig,
        *,
        logger: logging.Logger | None = None,
        trace_hook: TraceHook | None = None,
    ) -> None:
        config.validate()
        self._config = config
        self._telemetry = Telemetry(logger=logger, trace_hook=trace_hook)
        self._transport = HttpTransport(
            base_url=config.base_url,
            timeout_seconds=config.timeout_seconds,
            default_headers={
                "User-Agent": config.user_agent,
                **({"Authorization": f"Bearer {config.auth_token}"} if config.auth_token else {}),
                **({"X-Request-Id": config.request_id} if config.request_id else {}),
                **config.default_headers,
            },
            retry_policy=RetryPolicy(
                max_retries=config.max_retries,
                backoff_seconds=config.backoff_seconds,
                max_backoff_seconds=config.max_backoff_seconds,
            ),
            telemetry=self._telemetry,
            verify_ssl=config.verify_ssl,
            proxy_url=config.proxy_url,
            validate_response_schema=config.validate_response_schema,
        )

    def query(self, request: QueryRequest) -> Page:
        payload = request.to_payload()
        response = self._transport.post_json("v1/query", payload, idempotent=True)
        return Page(items=page_items(response), next_token=next_page_token(response))

    def stream_query(self, request: QueryRequest) -> Iterator[dict[str, object]]:
        page_token = request.page_token
        while True:
            current = QueryRequest(
                dataset=request.dataset,
                filters=request.filters,
                fields=request.fields,
                limit=request.limit,
                page_token=page_token,
            )
            page = self.query(current)
            for item in page.items:
                yield item
            if page.next_token is None:
                return
            page_token = page.next_token

    @classmethod
    def from_env(cls) -> "AtlasClient":
        """Build a client from BIJUX_ATLAS_* environment variables."""

        return cls(ClientConfig.from_env())

    def discover_runtime_info(self) -> dict[str, object]:
        """Discover runtime metadata from /version or fallback /health endpoint."""

        try:
            payload = self._transport.get_json("version", idempotent=True)
            payload["discovery_endpoint"] = "/version"
            return payload
        except Exception:  # pragma: no cover - fallback behavior
            payload = self._transport.get_json("health", idempotent=True)
            payload["discovery_endpoint"] = "/health"
            return payload

    def check_compatibility(self) -> dict[str, object]:
        """Check SDK/runtime compatibility using compatibility.json and runtime discovery."""

        matrix_path = pathlib.Path(__file__).with_name("compatibility.json")
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
        runtime = self.discover_runtime_info()
        runtime_version = str(runtime.get("version", runtime.get("runtime_version", "unknown")))
        client_major = __version__.split(".", maxsplit=1)[0]
        supported = matrix.get("client_major_support", {}).get(client_major, [])
        is_supported = any(runtime_version.startswith(prefix) for prefix in supported)
        result = {
            "client_version": __version__,
            "runtime_version": runtime_version,
            "supported_runtime_prefixes": supported,
            "is_supported": is_supported,
        }
        if not is_supported:
            warnings.warn(
                f"runtime version {runtime_version} is outside supported ranges for SDK {__version__}",
                RuntimeWarning,
                stacklevel=2,
            )
        return result
