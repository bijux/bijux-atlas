"""Top-level Atlas client."""

from __future__ import annotations

import logging
from collections.abc import Iterator

from .config import ClientConfig
from .http import HttpTransport
from .pagination import Page, next_page_token, page_items
from .query import QueryRequest
from .retry import RetryPolicy
from .telemetry import Telemetry, TraceHook


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
                **config.default_headers,
            },
            retry_policy=RetryPolicy(
                max_retries=config.max_retries,
                backoff_seconds=config.backoff_seconds,
            ),
            telemetry=self._telemetry,
        )

    def query(self, request: QueryRequest) -> Page:
        payload = request.to_payload()
        response = self._transport.post_json("v1/query", payload)
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
