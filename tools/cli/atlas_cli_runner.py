#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import pathlib
import subprocess
import sys
import time
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover - python 3.10 fallback
    tomllib = None  # type: ignore[assignment]


def _parse_simple_toml(raw: str) -> dict[str, object]:
    parsed: dict[str, object] = {}
    for line in raw.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        key, value = [part.strip() for part in line.split("=", 1)]
        if value.startswith('"') and value.endswith('"'):
            parsed[key] = value[1:-1]
        elif value.lower() in {"true", "false"}:
            parsed[key] = value.lower() == "true"
        else:
            parsed[key] = value
    return parsed


def default_config_path() -> pathlib.Path:
    xdg = os.getenv("XDG_CONFIG_HOME")
    if xdg:
        return pathlib.Path(xdg) / "bijux-dev-atlas" / "config.toml"
    return pathlib.Path.home() / ".config" / "bijux-dev-atlas" / "config.toml"


def load_config(path: pathlib.Path) -> dict[str, object]:
    if not path.exists():
        return {}
    if tomllib is not None:
        with path.open("rb") as fh:
            data = tomllib.load(fh)
    else:
        data = _parse_simple_toml(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError("cli.config invalid: expected table")
    return data


def apply_env_overrides(config: dict[str, object]) -> dict[str, object]:
    out = dict(config)
    if value := os.getenv("BIJUX_DEV_ATLAS_REPO_ROOT"):
        out["repo_root"] = value
    if value := os.getenv("BIJUX_DEV_ATLAS_OUTPUT_FORMAT"):
        out["output_format"] = value
    if value := os.getenv("BIJUX_DEV_ATLAS_QUIET"):
        out["quiet"] = value.lower() in {"1", "true", "yes", "on"}
    if value := os.getenv("BIJUX_DEV_ATLAS_PROFILE"):
        out["profile"] = value
    return out


def validate_config(config: dict[str, object]) -> None:
    allowed = {"repo_root", "output_format", "quiet", "profile"}
    unknown = set(config) - allowed
    if unknown:
        raise ValueError(f"cli.config unknown keys: {sorted(unknown)}")
    fmt = config.get("output_format")
    if fmt is not None and fmt not in {"human", "json", "both"}:
        raise ValueError("cli.config invalid output_format")
    quiet = config.get("quiet")
    if quiet is not None and not isinstance(quiet, bool):
        raise ValueError("cli.config quiet must be boolean")


def build_command(binary: str, config: dict[str, object], passthrough: list[str]) -> list[str]:
    cmd = [binary]
    if repo_root := config.get("repo_root"):
        cmd.extend(["--repo-root", str(repo_root)])
    if output_format := config.get("output_format"):
        cmd.extend(["--output-format", str(output_format)])
    if config.get("quiet") is True:
        cmd.append("--quiet")
    cmd.extend(passthrough)
    return cmd


from tools.cli.observability import ExecutionTelemetry, classify_error, emit_audit, emit_telemetry, emit_trace


def main() -> int:
    parser = argparse.ArgumentParser(description="Config-aware wrapper for bijux-dev-atlas")
    parser.add_argument("--binary", default="target/debug/bijux-dev-atlas")
    parser.add_argument("--config", type=pathlib.Path, default=default_config_path())
    parser.add_argument("--print-effective-config", action="store_true")
    parser.add_argument("--telemetry-out", type=pathlib.Path, default=pathlib.Path("artifacts/cli/command-telemetry.json"))
    parser.add_argument("--trace-out", type=pathlib.Path, default=pathlib.Path("artifacts/cli/command-trace.json"))
    parser.add_argument("--audit-out", type=pathlib.Path, default=pathlib.Path("artifacts/cli/command-audit.json"))
    parser.add_argument("args", nargs=argparse.REMAINDER)
    ns = parser.parse_args()

    config = apply_env_overrides(load_config(ns.config))
    validate_config(config)

    if ns.print_effective_config:
        for key in sorted(config):
            print(f"{key}={config[key]}")
        return 0

    cmd = build_command(ns.binary, config, ns.args)
    start = time.time()
    proc = subprocess.run(cmd, check=False, capture_output=True, text=True)
    end = time.time()

    telemetry = ExecutionTelemetry(command=cmd, started_at=start, finished_at=end, exit_code=proc.returncode)
    emit_telemetry(ns.telemetry_out, telemetry)
    emit_trace(ns.trace_out, "cli.command.completed", {"exit_code": proc.returncode, "duration_ms": telemetry.duration_ms})
    err_class = classify_error(proc.returncode, proc.stderr or "")
    emit_audit(ns.audit_out, "command_run", "success" if proc.returncode == 0 else "failure", {"classification": err_class})

    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)
    return proc.returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as err:
        print(str(err), file=sys.stderr)
        raise SystemExit(2)
