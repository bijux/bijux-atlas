from __future__ import annotations

import os
import sys


def main() -> int:
    ns = os.environ.get("ATLAS_NS") or os.environ.get("ATLAS_E2E_NAMESPACE") or ""
    if not ns:
        print("ATLAS_NS/ATLAS_E2E_NAMESPACE is empty", file=sys.stderr)
        return 1
    if ns.startswith("atlas-ops-") or ns == "atlas-e2e":
        return 0
    print(f"namespace must match atlas-ops-* or atlas-e2e (got: {ns})", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
