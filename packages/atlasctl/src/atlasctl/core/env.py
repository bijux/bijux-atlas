"""Centralized environment variable helpers."""

from __future__ import annotations

import os


def getenv(name: str, default: str | None = None) -> str | None:
    return os.environ.get(name, default)


def setdefault(name: str, value: str) -> str:
    return os.environ.setdefault(name, value)


def setenv(name: str, value: str) -> None:
    os.environ[name] = value
