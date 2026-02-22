#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
CONTRACT_PATH = ROOT / "ops" / "_meta" / "layer-contract.json"
ALLOWLIST = ROOT / "ops" / "_meta" / "stack-layer-literal-allowlist.txt"


def _load_literals() -> list[str]:
    obj = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))
    vals: set[str] = set()
    vals.update(str(v) for v in obj.get("namespaces", {}).values())
    vals.update(str(v) for v in obj.get("release_metadata", {}).get("defaults", {}).values())
    for svc in obj.get("services", {}).values():
        if isinstance(svc.get("service_name"), str):
            vals.add(svc["service_name"])
    return sorted(v for v in vals if v and v not in {"0.1.0"})


def main() -> int:
    literals = _load_literals()
    patterns = [re.compile(rf"\b{re.escape(x)}\b") for x in literals]
    allowed: set[Path] = set()
    if ALLOWLIST.exists():
        for raw in ALLOWLIST.read_text(encoding="utf-8").splitlines():
            line = raw.strip()
            if line and not line.startswith("#"):
                allowed.add((ROOT / line).resolve())

    errors: list[str] = []
    for path in sorted((ROOT / "ops/stack").rglob("*.sh")):
        if path.resolve() in allowed:
            continue
        text = path.read_text(encoding="utf-8")
        if "ops_layer_" in text or "ops_layer_contract_get" in text:
            continue
        for no, raw in enumerate(text.splitlines(), start=1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            if "ops_layer_" in line or "ops_layer_contract_get" in line:
                continue
            for pat in patterns:
                if pat.search(line):
                    errors.append(
                        f"{path.relative_to(ROOT)}:{no}: stack layer literal must come from ops/_meta/layer-contract.json"
                    )
                    break

    if errors:
        print("stack layer literal lint failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("stack layer literal lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
