from __future__ import annotations

import argparse

from atlasctl.commands.check.parser import configure_check_parser


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="atlasctl")
    sub = parser.add_subparsers(dest="cmd")
    configure_check_parser(sub)
    return parser


def test_check_list_supports_domains_tags_and_suites_flags() -> None:
    parser = _build_parser()
    ns = parser.parse_args(["check", "list", "--domains"])
    assert ns.domains is True
    ns = parser.parse_args(["check", "list", "--tags"])
    assert ns.tags is True
    ns = parser.parse_args(["check", "list", "--suites"])
    assert ns.suites is True
