from __future__ import annotations

from dataclasses import dataclass, field
from enum import IntEnum


class ExitCode(IntEnum):
    OK = 0
    FAIL = 1
    USAGE = 2
    CONFIG = 3
    PREREQ = 4


@dataclass(frozen=True)
class CheckError:
    code: str
    message: str
    path: str | None = None


@dataclass(frozen=True)
class CheckResult:
    check_id: str
    status: str
    errors: tuple[CheckError, ...] = ()
    warnings: tuple[str, ...] = ()
    metrics: dict[str, object] = field(default_factory=dict)


@dataclass(frozen=True)
class CommandResult:
    command: str
    exit_code: int | ExitCode
    status: str
    output: str = ""
    errors: tuple[str, ...] = ()


__all__ = ["CheckError", "CheckResult", "CommandResult", "ExitCode"]
