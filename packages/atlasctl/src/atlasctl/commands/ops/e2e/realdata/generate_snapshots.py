from __future__ import annotations

import hashlib
import json
import os
import sys
import urllib.request
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import env_root


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    root = env_root()
    base_url = os.environ.get("ATLAS_E2E_BASE_URL", "http://127.0.0.1:18080")
    queries_json = root / "ops/e2e/realdata/canonical_queries.json"
    out_json = Path(argv[0]) if argv else root / "artifacts/ops/e2e/realdata/release110_snapshot.generated.json"
    out_json.parent.mkdir(parents=True, exist_ok=True)

    queries = json.loads(queries_json.read_text(encoding="utf-8"))["queries"]
    entries: list[dict[str, object]] = []
    for q in queries:
        req = urllib.request.Request(base_url + q, method="GET")
        try:
            with urllib.request.urlopen(req, timeout=20) as response:
                body = response.read()
                status = response.status
        except Exception as exc:  # parity with shell script behavior
            body = str(exc).encode()
            status = 599
        entries.append(
            {
                "query": q,
                "status": status,
                "sha256": hashlib.sha256(body).hexdigest(),
                "size": len(body),
            }
        )

    payload = {
        "schema_version": 1,
        "generated_from": "ops/e2e/realdata/canonical_queries.json",
        "entries": entries,
    }
    out_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out_json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
