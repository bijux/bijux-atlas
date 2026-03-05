"""HTTP layer used by bijux-atlas."""

from __future__ import annotations

import json
import time
from dataclasses import dataclass

import requests

from .errors import AtlasApiError, AtlasRetryExhaustedError
from .retry import RetryPolicy
from .telemetry import Telemetry


@dataclass(slots=True)
class HttpTransport:
    """HTTP transport with timeout and retry support."""

    base_url: str
    timeout_seconds: float
    default_headers: dict[str, str]
    retry_policy: RetryPolicy
    telemetry: Telemetry
    verify_ssl: bool = True
    proxy_url: str | None = None
    validate_response_schema: bool = False

    def post_json(self, path: str, payload: dict[str, object], *, idempotent: bool = True) -> dict[str, object]:
        return self._json_request("POST", path, payload, idempotent=idempotent)

    def get_json(self, path: str, *, idempotent: bool = True) -> dict[str, object]:
        return self._json_request("GET", path, None, idempotent=idempotent)

    def _json_request(
        self,
        method: str,
        path: str,
        payload: dict[str, object] | None,
        *,
        idempotent: bool,
    ) -> dict[str, object]:
        url = f"{self.base_url.rstrip('/')}/{path.lstrip('/')}"
        timeout = self.timeout_seconds
        proxies = {"http": self.proxy_url, "https": self.proxy_url} if self.proxy_url else None

        last_error: Exception | None = None
        for attempt in range(self.retry_policy.max_retries + 1):
            self.telemetry.emit_trace("atlas.http.request.start", url=url, attempt=attempt)
            try:
                response = requests.request(
                    method=method,
                    url=url,
                    json=payload,
                    headers={
                        "Content-Type": "application/json",
                        **self.default_headers,
                    },
                    timeout=timeout,
                    verify=self.verify_ssl,
                    proxies=proxies,
                )
                status_code = response.status_code
                body_text = response.text
                request_id = response.headers.get("x-request-id")
                trace_id = response.headers.get("x-trace-id")
                self.telemetry.emit_log(
                    "atlas.http.request.complete",
                    url=url,
                    status_code=status_code,
                    attempt=attempt,
                )
                self.telemetry.emit_trace(
                    "atlas.http.request.complete",
                    url=url,
                    status_code=status_code,
                    attempt=attempt,
                )
                if status_code >= 400:
                    raise AtlasApiError(
                        status_code=status_code,
                        body=body_text,
                        request_id=request_id,
                        trace_id=trace_id,
                    )
                decoded = response.json()
                if self.validate_response_schema and not isinstance(decoded, dict):
                    raise AtlasApiError(
                        status_code=500,
                        body="response schema mismatch: expected object payload",
                        request_id=request_id,
                        trace_id=trace_id,
                    )
                if isinstance(decoded, dict):
                    return decoded
                return {"value": decoded}
            except AtlasApiError as error:
                last_error = error
                if self.retry_policy.should_retry(
                    attempt,
                    error.status_code,
                    None,
                    idempotent=idempotent,
                ):
                    time.sleep(self.retry_policy.backoff_delay(attempt))
                    continue
                raise
            except (requests.RequestException, json.JSONDecodeError) as error:
                last_error = error
                if self.retry_policy.should_retry(
                    attempt,
                    None,
                    error,
                    idempotent=idempotent,
                ):
                    time.sleep(self.retry_policy.backoff_delay(attempt))
                    continue
                raise AtlasRetryExhaustedError(attempts=attempt + 1, last_error=error) from error

        raise AtlasRetryExhaustedError(attempts=self.retry_policy.max_retries + 1, last_error=last_error)
