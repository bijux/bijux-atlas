from __future__ import annotations

import argparse
from collections.abc import Callable
from typing import TypeVar

from atlasctl.core.context import RunContext

Runner = Callable[[RunContext, str, str], int]


def run_group_action(ctx: RunContext, ns: argparse.Namespace, action_attr: str, runner: Runner, *, report_attr: str = "report") -> int:
    action = str(getattr(ns, action_attr, "") or "").strip()
    report = str(getattr(ns, report_attr, "text") or "text")
    return runner(ctx, action, report)
