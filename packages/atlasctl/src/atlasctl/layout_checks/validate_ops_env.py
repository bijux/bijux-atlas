#!/usr/bin/env python3
# Purpose: validate canonical ops environment against schema and print resolved values.
# Inputs: configs/ops/env.schema.json and current process environment.
# Outputs: non-zero exit on invalid env; optional export/json print.
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from urllib.parse import urlparse


ROOT = Path(__file__).resolve().parents[3]


def _validate_value(name: str, spec: dict[str, object], value: str) -> str | None:
    kind = spec.get("type")
    if kind == "string":
        return None if value else f"{name} must not be empty"
    if kind == "url":
        parsed = urlparse(value)
        if parsed.scheme not in ("http", "https") or not parsed.netloc:
            return f"{name} must be an absolute http(s) URL"
        return None
    if kind == "k8s_name":
        if re.fullmatch(r"[a-z0-9]([-a-z0-9]*[a-z0-9])?", value):
            return None
        return f"{name} must match Kubernetes DNS label format"
    if kind == "path":
        if not value:
            return f"{name} must not be empty"
        if spec.get("must_exist"):
            p = (ROOT / value).resolve()
            if not p.exists():
                return f"{name} points to missing path: {value}"
        return None
    if kind == "integer":
        if re.fullmatch(r"-?[0-9]+", value):
            return None
        return f"{name} must be an integer"
    return f"{name} has unsupported type in schema: {kind}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--schema", required=True)
    parser.add_argument("--print", action="store_true", dest="print_env")
    parser.add_argument("--format", choices=("env", "json"), default="env")
    args = parser.parse_args()

    schema_path = (ROOT / args.schema).resolve()
    data = json.loads(schema_path.read_text(encoding="utf-8"))
    variables: dict[str, dict[str, object]] = data["variables"]

    resolved: dict[str, str] = {}
    for name, spec in variables.items():
        raw = os.environ.get(name)
        if raw is not None and raw != "":
            resolved[name] = raw
            continue
        default_from = spec.get("default_from")
        if isinstance(default_from, str) and default_from in resolved:
            resolved[name] = resolved[default_from]
            continue
        default = spec.get("default")
        if isinstance(default, (str, int)):
            resolved[name] = str(default)
            continue
        resolved[name] = ""

    # Expand ${VAR} placeholders in resolved values once all defaults are present.
    var_ref = re.compile(r"\$\{([A-Z0-9_]+)\}")
    for _ in range(3):
        changed = False
        for name, value in list(resolved.items()):
            expanded = var_ref.sub(lambda m: resolved.get(m.group(1), ""), value)
            if expanded != value:
                resolved[name] = expanded
                changed = True
        if not changed:
            break

    errors: list[str] = []
    for name, spec in variables.items():
        error = _validate_value(name, spec, resolved[name])
        if error:
            errors.append(error)

    if errors:
        for err in errors:
            print(f"ops env contract violation: {err}", file=sys.stderr)
        return 1

    if args.print_env:
        if args.format == "json":
            print(json.dumps({name: resolved[name] for name in sorted(resolved)}, indent=2, sort_keys=True))
        else:
            for name in sorted(resolved):
                print(f"{name}={resolved[name]}")
    else:
        print("ops env contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
