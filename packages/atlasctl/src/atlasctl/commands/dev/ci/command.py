from __future__ import annotations

import argparse

from ...ci.command import configure_ci_parser as configure_ci_group_parser
from ...ci.command import run_ci_command as run_ci_group_command
from ....core.context import RunContext


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    # DEV compatibility shim: canonical CI surface lives at atlasctl.commands.ci.command.
    return run_ci_group_command(ctx, ns)


def configure_ci_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    # DEV compatibility shim: parser shape is defined by canonical CI group parser.
    configure_ci_group_parser(sub)
