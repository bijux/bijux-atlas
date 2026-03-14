"""Query API wrapper payloads."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass(slots=True)
class QueryRequest:
    """Represents a dataset query request."""

    dataset: str
    filters: dict[str, Any] = field(default_factory=dict)
    fields: list[str] = field(default_factory=list)
    limit: int | None = None
    page_token: str | None = None

    def to_payload(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "dataset": self.dataset,
            "filters": self.filters,
            "fields": self.fields,
        }
        if self.limit is not None:
            payload["limit"] = self.limit
        if self.page_token is not None:
            payload["page_token"] = self.page_token
        return payload
