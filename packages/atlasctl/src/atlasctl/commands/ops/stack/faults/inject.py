from __future__ import annotations

import sys

from atlasctl.commands.ops.stack.faults import block_minio, cpu_throttle, fill_node_disk, throttle_network, toxiproxy_latency


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    action = args[0] if args else ""
    rest = args[1:]
    if action == "block-minio":
        return block_minio.main([rest[0] if rest else "on"])
    if action == "toxiproxy-latency":
        latency = rest[0] if len(rest) > 0 else "250"
        jitter = rest[1] if len(rest) > 1 else "25"
        return toxiproxy_latency.main([latency, jitter])
    if action == "throttle-network":
        return throttle_network.main([rest[0] if rest else "256"])
    if action == "cpu-throttle":
        return cpu_throttle.main()
    if action == "fill-node-disk":
        return fill_node_disk.main([rest[0] if rest else "fill"])
    print(
        "usage: inject.py {block-minio|toxiproxy-latency|throttle-network|cpu-throttle|fill-node-disk} [args...]",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
