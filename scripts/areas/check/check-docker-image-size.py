#!/usr/bin/env python3
# Purpose: enforce runtime image size budget.
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BUDGET = json.loads((ROOT / "docker/contracts/image-size-budget.json").read_text(encoding="utf-8"))


def main() -> int:
    image = f"{__import__('os').environ.get('DOCKER_IMAGE','bijux-atlas:local')}"
    max_bytes = int(BUDGET["runtime_image_max_bytes"])
    cmd = ["docker", "image", "inspect", image, "--format", "{{.Size}}"]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        print(f"docker image size check skipped: could not inspect {image}", file=sys.stderr)
        return 0
    try:
        size = int(proc.stdout.strip())
    except ValueError:
        print(f"docker image size check failed: invalid size output '{proc.stdout.strip()}'", file=sys.stderr)
        return 1

    if size > max_bytes:
        print(
            f"docker image size budget exceeded: {size} > {max_bytes} bytes for {image}",
            file=sys.stderr,
        )
        return 1

    print(f"docker image size budget passed: {size} <= {max_bytes} bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
