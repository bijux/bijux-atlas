from __future__ import annotations

import os
import subprocess
import sys
import time


def _parse_ts(ts: str) -> int:
    if not ts:
        return 0
    for fmt in ("%Y-%m-%dT%H:%M:%SZ",):
        try:
            import datetime as dt

            return int(dt.datetime.strptime(ts, fmt).replace(tzinfo=dt.timezone.utc).timestamp())
        except Exception:
            pass
    try:
        return int(subprocess.check_output(["date", "-d", ts, "+%s"], text=True).strip())
    except Exception:
        return 0


def main() -> int:
    age_min = int(os.environ.get("OPS_STALE_NAMESPACE_MINUTES", "240"))
    now = int(time.time())
    ns_list = subprocess.run(
        ["kubectl", "get", "ns", "-o", "jsonpath={range .items[*]}{.metadata.name}{\"\\n\"}{end}"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()
    for ns in ns_list:
        if not ns or not ns.startswith("atlas-ops-"):
            continue
        ts = subprocess.run(
            ["kubectl", "get", "ns", ns, "-o", "jsonpath={.metadata.creationTimestamp}"],
            capture_output=True,
            text=True,
        ).stdout.strip()
        created = _parse_ts(ts)
        if created == 0:
            continue
        age = (now - created) // 60
        if age >= age_min:
            print(f"deleting stale namespace: {ns} (age {age}m)")
            subprocess.run(["kubectl", "delete", "ns", ns, "--ignore-not-found"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
