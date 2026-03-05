"""HTTP layer used by bijux-atlas."""

from __future__ import annotations

import json
import ssl
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from urllib.parse import urlparse

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

    def _build_opener(self) -> urllib.request.OpenerDirector:
        handlers: list[urllib.request.BaseHandler] = []
        ssl_context = self._ssl_context()
        if ssl_context is not None:
            handlers.append(urllib.request.HTTPSHandler(context=ssl_context))
        if self.proxy_url:
            parsed = urlparse(self.proxy_url)
            proxy_map = {parsed.scheme: self.proxy_url}
            handlers.append(urllib.request.ProxyHandler(proxy_map))
        return urllib.request.build_opener(*handlers)

    def _ssl_context(self) -> ssl.SSLContext | None:
        if self.verify_ssl:
            return None
        return ssl._create_unverified_context()  # noqa: SLF001

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
        data = json.dumps(payload).encode("utf-8") if payload is not None else None
        opener = self._build_opener()

        last_error: Exception | None = None
        for attempt in range(self.retry_policy.max_retries + 1):
            self.telemetry.emit_trace("atlas.http.request.start", url=url, attempt=attempt)
            request = urllib.request.Request(
                url,
                data=data,
                headers={
                    "Content-Type": "application/json",
                    **self.default_headers,
                },
                method=method,
            )
            try:
                with opener.open(request, timeout=self.timeout_seconds) as response:
                    body_bytes = response.read()
                    status_code = response.getcode()
                    body_text = body_bytes.decode("utf-8")
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
                    decoded = json.loads(body_text)
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
            except urllib.error.HTTPError as error:
                body_text = error.read().decode("utf-8")
                api_error = AtlasApiError(
                    status_code=error.code,
                    body=body_text,
                    request_id=error.headers.get("x-request-id") if error.headers else None,
                    trace_id=error.headers.get("x-trace-id") if error.headers else None,
                )
                last_error = api_error
                if self.retry_policy.should_retry(attempt, error.code, None, idempotent=idempotent):
                    time.sleep(self.retry_policy.backoff_delay(attempt))
                    continue
                raise api_error
            except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
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
