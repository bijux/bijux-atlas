from __future__ import annotations

import json
from typing import Any

from .model import CheckResult


def results_as_rows(results: list[CheckResult]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for result in sorted(results, key=lambda row: row.canonical_key):
        rows.append(
            {
                "id": str(result.id),
                "domain": str(result.domain),
                "status": str(result.status),
                "category": str(result.category),
                "result_code": str(result.result_code),
                "violations": [
                    {
                        "code": str(item.code),
                        "message": item.message,
                        "hint": item.hint,
                        "path": item.path,
                        "line": item.line,
                        "column": item.column,
                        "severity": str(item.severity),
                    }
                    for item in result.violations
                ],
                "warnings": list(result.warnings),
                "metrics": dict(result.metrics),
                "evidence_paths": list(result.evidence_paths),
            }
        )
    return rows


def to_json(results: list[CheckResult]) -> str:
    return json.dumps({"rows": results_as_rows(results)}, indent=2, sort_keys=True)


__all__ = ["results_as_rows", "to_json"]
