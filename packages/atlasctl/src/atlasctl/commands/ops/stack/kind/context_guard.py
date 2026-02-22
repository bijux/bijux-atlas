from __future__ import annotations

import os
import subprocess
import sys


def main() -> int:
    if os.environ.get("ALLOW_NON_KIND", "0") == "1" or os.environ.get("I_KNOW_WHAT_I_AM_DOING", "0") == "1":
        return 0
    proc = subprocess.run(["kubectl", "config", "current-context"], capture_output=True, text=True)
    ctx = (proc.stdout or "").strip()
    if not ctx:
        print("kubectl context is not set", file=sys.stderr)
        return 1
    if ctx.startswith("kind-") or "kind" in ctx:
        return 0
    print(f"refusing non-kind context '{ctx}' (set I_KNOW_WHAT_I_AM_DOING=1 to override)", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
