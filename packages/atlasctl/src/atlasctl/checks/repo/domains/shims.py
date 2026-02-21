"""Repository shim and compatibility checks."""

from __future__ import annotations

from .legacy_native import check_root_bin_shims, check_script_shim_expiry, check_script_shims_minimal

__all__ = ["check_root_bin_shims", "check_script_shim_expiry", "check_script_shims_minimal"]
