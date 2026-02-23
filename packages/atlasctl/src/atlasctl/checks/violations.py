from __future__ import annotations

from .model import ResultCode, Severity, Violation


def v(
    code: ResultCode | str,
    message: str,
    *,
    hint: str = "",
    path: str = "",
    line: int = 0,
    column: int = 0,
    severity: Severity = Severity.ERROR,
) -> Violation:
    return Violation(
        code=str(code),
        message=message,
        hint=hint,
        path=path,
        line=line,
        column=column,
        severity=severity,
    )


__all__ = ["v"]
