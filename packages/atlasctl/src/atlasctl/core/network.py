"""Centralized network boundary helpers."""

from __future__ import annotations

import urllib.request


def http_get(url: str, timeout_seconds: int = 5) -> tuple[int, str]:
    req = urllib.request.Request(url, method="GET")
    with urllib.request.urlopen(req, timeout=timeout_seconds) as resp:  # nosec - caller controls endpoint
        return int(resp.status), resp.read().decode("utf-8", errors="replace")
