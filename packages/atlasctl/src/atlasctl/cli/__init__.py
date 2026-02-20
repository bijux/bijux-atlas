"""Atlasctl CLI package."""

from __future__ import annotations

from importlib import import_module

__all__ = ["build_parser", "main"]


def build_parser():
    return import_module("atlasctl.cli.main").build_parser()


def main(argv=None):
    return import_module("atlasctl.cli.main").main(argv)
