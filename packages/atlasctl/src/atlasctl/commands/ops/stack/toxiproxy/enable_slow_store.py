from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    root = Path(__file__).resolve().parents[7]
    latency_ms = args[0] if len(args) > 0 else "1200"
    jitter_ms = args[1] if len(args) > 1 else "200"
    subprocess.run(
        [
            sys.executable,
            str(root / "packages/atlasctl/src/atlasctl/commands/ops/stack/faults/inject.py"),
            "toxiproxy-latency",
            latency_ms,
            jitter_ms,
        ],
        check=True,
        cwd=root,
    )
    print("slow store mode enabled via toxiproxy latency")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
