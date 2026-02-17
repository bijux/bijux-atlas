#!/usr/bin/env python3
import sys
from html.parser import HTMLParser
from pathlib import Path

class LinkParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
    def handle_starttag(self, tag, attrs):
        if tag != "a":
            return
        href = dict(attrs).get("href")
        if href:
            self.links.append(href)

def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_mkdocs_site_links.py <site_dir>", file=sys.stderr)
        return 2
    site = Path(sys.argv[1])
    if not site.exists():
        print(f"site dir missing: {site}", file=sys.stderr)
        return 2

    errors = []
    for html in site.rglob("*.html"):
        if html.name == "404.html":
            continue
        p = LinkParser()
        p.feed(html.read_text(encoding="utf-8"))
        for href in p.links:
            if href.startswith(("http://", "https://", "mailto:", "#")):
                continue
            if href.startswith("/"):
                target_root = href.split("#", 1)[0].lstrip("/")
                resolved = (site / target_root).resolve()
                if resolved.is_dir():
                    resolved = resolved / "index.html"
                elif resolved.suffix == "":
                    resolved = resolved.with_suffix(".html")
                if not resolved.exists():
                    errors.append(f"{html}: broken site-root link -> {href}")
                continue
            target = href.split("#", 1)[0]
            if not target:
                continue
            resolved = (html.parent / target).resolve()
            if resolved.is_dir():
                resolved = resolved / "index.html"
            if not resolved.exists():
                errors.append(f"{html}: broken link -> {href}")
    if errors:
        print("mkdocs output link-check failed:", file=sys.stderr)
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print("mkdocs output link-check passed")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
