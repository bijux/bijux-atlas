"""HTTP layer used by bijux-atlas."""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.request
from dataclasses import dataclass

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

    def post_json(self, path: str, payload: dict[str, object]) -> dict[str, object]:
        url = f"{self.base_url.rstrip('/')}/{path.lstrip('/')}"
        data = json.dumps(payload).encode("utf-8")

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
                method="POST",
            )
            try:
                with urllib.request.urlopen(request, timeout=self.timeout_seconds) as response:
                    body_bytes = response.read()
                    status_code = response.getcode()
                    body_text = body_bytes.decode("utf-8")
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
                        raise AtlasApiError(status_code=status_code, body=body_text)
                    decoded = json.loads(body_text)
                    if isinstance(decoded, dict):
                        return decoded
                    return {"value": decoded}
            except AtlasApiError as error:
                last_error = error
                if self.retry_policy.should_retry(attempt, error.status_code, None):
                    time.sleep(self.retry_policy.backoff_delay(attempt))
                    continue
                raise
            except urllib.error.HTTPError as error:
                body_text = error.read().decode("utf-8")
                api_error = AtlasApiError(status_code=error.code, body=body_text)
                last_error = api_error
                if self.retry_policy.should_retry(attempt, error.code, None):
                    time.sleep(self.retry_policy.backoff_delay(attempt))
                    continue
                raise api_error
            except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
                last_error = error
                if self.retry_policy.should_retry(attempt, None, error):
                    time.sleep(self.retry_policy.backoff_delay(attempt))
                    continue
                raise AtlasRetryExhaustedError(attempts=attempt + 1, last_error=error) from error

        raise AtlasRetryExhaustedError(attempts=self.retry_policy.max_retries + 1, last_error=last_error)
