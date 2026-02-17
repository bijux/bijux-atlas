#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

REQUIRED_HEADINGS = [
    "what",
    "why",
    "scope",
    "non-goals",
    "contracts",
    "failure modes",
    "how to verify",
    "see also",
]

BANNED_MARKETING = re.compile(r"\b(elite|reference-grade|world-class|best-in-class)\b", re.IGNORECASE)
BANNED_VAGUE = re.compile(r"\b(should|might|could|maybe|perhaps)\b", re.IGNORECASE)
BANNED_INCLUSIVE = re.compile(r"\b(whitelist|blacklist|master|slave)\b", re.IGNORECASE)
HEADING_RE = re.compile(r"^##\s+(.+?)\s*$", re.MULTILINE)
CODEBLOCK_RE = re.compile(r"```(?:bash|sh)\n(.*?)```", re.DOTALL)
LINK_RE = re.compile(r"\[[^\]]+\]\([^\)]+\)")


def section_body(text: str, heading: str) -> str:
    pattern = re.compile(rf"^##\s+{re.escape(heading)}\s*$", re.IGNORECASE | re.MULTILINE)
    m = pattern.search(text)
    if not m:
        return ""
    start = m.end()
    next_h = re.compile(r"^##\s+", re.MULTILINE).search(text, start)
    end = next_h.start() if next_h else len(text)
    return text[start:end].strip()


def lint_file(path: pathlib.Path) -> list[str]:
    rel = path.relative_to(ROOT)
    text = path.read_text(encoding="utf-8")
    errors: list[str] = []

    if rel.as_posix() == "docs/product/reference-grade-checklist.md":
        allow_marketing = True
    else:
        allow_marketing = False

    headings = {h.strip().lower() for h in HEADING_RE.findall(text)}
    for req in REQUIRED_HEADINGS:
        if req not in headings:
            errors.append(f"{rel}: missing required heading '## {req.title()}'")

    if not allow_marketing and BANNED_MARKETING.search(text):
        errors.append(f"{rel}: contains banned marketing adjectives")

    if BANNED_VAGUE.search(text):
        errors.append(f"{rel}: contains vague verbs (should/might/could/maybe/perhaps)")

    if BANNED_INCLUSIVE.search(text):
        errors.append(f"{rel}: contains non-inclusive terminology")

    # examples mandatory for contract docs
    examples = section_body(text, "Examples")
    if not examples:
        errors.append(f"{rel}: missing required heading '## Examples'")
    else:
        if "```" not in examples:
            errors.append(f"{rel}: examples section must include fenced code block")

    # runnable command snippets + expected output shape (examples section only)
    has_expected = "expected output" in examples.lower()
    for block in CODEBLOCK_RE.findall(examples):
        lines = [ln.strip() for ln in block.splitlines() if ln.strip()]
        if not lines:
            continue
        if not all(ln.startswith("$") for ln in lines):
            errors.append(f"{rel}: shell code blocks must include full commands prefixed with '$'")
            break
    if CODEBLOCK_RE.search(examples) and not has_expected:
        errors.append(f"{rel}: command snippets require an 'Expected output' description")

    see_also = section_body(text, "See also")
    links = LINK_RE.findall(see_also)
    if not (3 <= len(links) <= 8):
        errors.append(f"{rel}: 'See also' must contain 3-8 links")
    if "terms-glossary.md" not in see_also:
        errors.append(f"{rel}: 'See also' must include glossary link")

    if "- Owner:" not in text:
        errors.append(f"{rel}: missing owner header ('- Owner:')")

    return errors


def main() -> int:
    targets = sorted((DOCS / "contracts").rglob("*.md"))
    all_errors: list[str] = []
    for file in targets:
        all_errors.extend(lint_file(file))

    if all_errors:
        for err in all_errors:
            print(err, file=sys.stderr)
        return 1

    print("doc contracts lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())