"""Core check data model and execution primitives."""

from .base import Check, CheckCategory, CheckDef, CheckFunc, CheckResult, Severity


def run_command_checks(*args: object, **kwargs: object):  # noqa: ANN002, ANN003
    from .execution import run_command_checks as _run_command_checks

    return _run_command_checks(*args, **kwargs)


def run_function_checks(*args: object, **kwargs: object):  # noqa: ANN002, ANN003
    from .execution import run_function_checks as _run_function_checks

    return _run_function_checks(*args, **kwargs)


class CommandCheckDef:  # compatibility alias resolved lazily
    def __new__(cls, *args: object, **kwargs: object):  # noqa: ANN002, ANN003
        from .execution import CommandCheckDef as _CommandCheckDef

        return _CommandCheckDef(*args, **kwargs)

__all__ = [
    "Check",
    "CheckCategory",
    "CheckDef",
    "CheckFunc",
    "CheckResult",
    "Severity",
    "CommandCheckDef",
    "run_command_checks",
    "run_function_checks",
]
