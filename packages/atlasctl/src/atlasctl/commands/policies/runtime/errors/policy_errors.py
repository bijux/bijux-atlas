from __future__ import annotations


class PolicyError(Exception):
    code = "POL000"

    def __init__(self, message: str) -> None:
        super().__init__(message)
        self.message = message


class BudgetMetricError(PolicyError):
    code = "POL001"
