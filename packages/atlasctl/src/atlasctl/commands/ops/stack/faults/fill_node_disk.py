from __future__ import annotations

import os
import subprocess
import sys


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    cluster = os.environ.get("ATLAS_E2E_CLUSTER_NAME", "bijux-atlas-e2e")
    node = f"{cluster}-control-plane"
    size_mb = os.environ.get("FILL_SIZE_MB", "512")
    mode = args[0] if args else "fill"
    file_path = "/var/tmp/atlas-disk-pressure.img"
    if os.environ.get("OPS_DRY_RUN", "0") == "1":
        print(f"DRY-RUN disk-pressure mode={mode} node={node} size_mb={size_mb}")
        return 0
    if mode == "clean":
        subprocess.run(["docker", "exec", node, "sh", "-c", f"rm -f {file_path}"], check=True)
        print("disk pressure file removed")
        return 0
    subprocess.run(
        ["docker", "exec", node, "sh", "-c", f"dd if=/dev/zero of={file_path} bs=1m count={size_mb} status=none"],
        check=True,
    )
    print(f"disk pressure simulated: {size_mb} MiB on {node}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
