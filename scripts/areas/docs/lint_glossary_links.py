#!/usr/bin/env python3
"""
Purpose: Ensure glossary terms are referenced in docs.
Inputs: docs/_style/terms-glossary.md and docs/**/*.md
Outputs: lint status.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"
GLOSSARY = DOCS / "_style" / "terms-glossary.md"


def parse_terms() -> list[str]:
    text = GLOSSARY.read_text(encoding="utf-8")
    terms: list[str] = []
    for line in text.splitlines():
        m = re.match(r"- `([^`]+)`:", line.strip())
        if m:
            terms.append(m.group(1))
    return terms


def main() -> int:
    terms = parse_terms()
    corpus_parts = []
    for path in DOCS.rglob("*.md"):
        if path == GLOSSARY:
            continue
        corpus_parts.append(path.read_text(encoding="utf-8"))
    corpus = "\n".join(corpus_parts)

    missing = [term for term in terms if re.search(rf"\b{re.escape(term)}\b", corpus) is None]
    if missing:
        print("glossary link lint failed; missing term usage:")
        for term in missing:
            print(f"- {term}")
        return 1
    print("glossary link lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
