"""Atlasctl stack package."""
from .build_stack_report import main as build_main
from .validate_stack_report import main as validate_main

__all__ = ["build_main", "validate_main"]
