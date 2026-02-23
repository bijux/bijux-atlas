"""CLI parsing boundary for atlasctl app bootstrap."""

from __future__ import annotations

from dataclasses import dataclass

from ..cli.main import build_parser


@dataclass(frozen=True)
class CliInvocation:
    raw_argv: tuple[str, ...]
    namespace: object


def parse_cli_invocation(argv: list[str] | None = None) -> CliInvocation:
    raw_argv = tuple(argv or [])
    parser = build_parser()
    ns = parser.parse_args(argv)
    return CliInvocation(raw_argv=raw_argv, namespace=ns)
