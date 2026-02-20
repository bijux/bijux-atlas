"""Shared argparse helpers for common atlasctl flags."""

from __future__ import annotations

import argparse


def add_json_flag(parser: argparse.ArgumentParser, help_text: str = "emit JSON output") -> None:
    parser.add_argument("--json", action="store_true", help=help_text)


def add_report_flag(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--report", choices=["text", "json"], default="text")
