#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    caps = json.loads((root / "configs/ops/hpa-safety-caps.json").read_text(encoding="utf-8"))
    default_cap = int(caps.get("default_max_replicas_cap", 0))
    profile_caps = caps.get("profile_caps", {})
    for values in sorted((root / "ops/k8s/values").glob("*.yaml")):
        lines = values.read_text(encoding="utf-8").splitlines()
        max_replicas = None
        in_hpa = False
        for line in lines:
            if line.startswith("hpa:"):
                in_hpa = True
                continue
            if in_hpa and line and not line.startswith(" "):
                in_hpa = False
            if in_hpa and "maxReplicas:" in line:
                try:
                    max_replicas = int(line.split(":", 1)[1].strip())
                except ValueError:
                    pass
                break
        if max_replicas is None:
            continue
        profile = values.stem
        cap = int(profile_caps.get(profile, default_cap))
        if max_replicas > cap:
            print(
                f"failure_mode: hpa_max_replicas_cap_exceeded profile={profile} "
                f"maxReplicas={max_replicas} cap={cap}"
            )
            return 1
    print("hpa max replicas cap contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
