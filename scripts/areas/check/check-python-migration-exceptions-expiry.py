#!/usr/bin/env python3
from __future__ import annotations

import sys

from python_migration_exceptions import expired_exceptions


def main() -> int:
    expired = expired_exceptions()
    if expired:
        print("python migration exceptions have expired:", file=sys.stderr)
        for entry in expired:
            print(
                f"- {entry.id} kind={entry.kind} owner={entry.owner} expires_on={entry.expires_on} issue={entry.issue}",
                file=sys.stderr,
            )
        return 1
    print("python migration exceptions expiry check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
