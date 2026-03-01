#!/usr/bin/env python3
import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path


LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
IMAGE_RE = re.compile(r"!\[[^\]]*\]\(([^)]+)\)")


def markdown_files(root: Path):
    for path in sorted(root.rglob("*.md")):
        rel = path.relative_to(root)
        if rel.parts[:1] in {("_generated",), ("_drafts",), ("_nav",)}:
            continue
        yield path


def load_allowlist(path: Path):
    payload = json.loads(path.read_text())
    return [entry["pattern"] for entry in payload.get("entries", [])]


def is_allowed(target: str, allowlist):
    return any(target.startswith(pattern) for pattern in allowlist)


def external_targets(text: str):
    targets = []
    for regex in (LINK_RE, IMAGE_RE):
        for target in regex.findall(text):
            cleaned = target.split("#", 1)[0].strip()
            if cleaned.startswith("http://") or cleaned.startswith("https://"):
                targets.append(cleaned)
    return sorted(set(targets))


def probe(url: str):
    req = urllib.request.Request(
        url,
        method="HEAD",
        headers={"User-Agent": "bijux-atlas-docs-link-check/1"},
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as response:
            return response.status < 400, response.status
    except urllib.error.HTTPError as err:
        if err.code in {405, 501}:
            fallback = urllib.request.Request(
                url,
                method="GET",
                headers={"User-Agent": "bijux-atlas-docs-link-check/1"},
            )
            with urllib.request.urlopen(fallback, timeout=10) as response:
                return response.status < 400, response.status
        return False, err.code
    except Exception as err:  # pragma: no cover - CI diagnostic path
        return False, str(err)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--docs-root", required=True)
    parser.add_argument("--allowlist", required=True)
    args = parser.parse_args()

    docs_root = Path(args.docs_root).resolve()
    allowlist = load_allowlist(Path(args.allowlist).resolve())

    violations = []
    seen = {}
    for path in markdown_files(docs_root):
        for target in external_targets(path.read_text()):
            seen.setdefault(target, []).append(path.relative_to(docs_root))

    for target, paths in sorted(seen.items()):
        if is_allowed(target, allowlist):
            continue
        ok, detail = probe(target)
        if not ok:
            refs = ", ".join(str(p) for p in paths[:3])
            violations.append(f"{target} failed external link check ({detail}); referenced from {refs}")

    if violations:
        sys.stderr.write("\n".join(violations) + "\n")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
