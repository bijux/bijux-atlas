#!/usr/bin/env python3
"""
Purpose: Enforce documentation depth rubric.
Inputs: docs/**/*.md
Outputs: artifacts/docs/depth-report.md and process exit code.
"""
from __future__ import annotations

import re
import os
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
ARTIFACTS = ROOT / "artifacts" / "docs"
REPORT = ARTIFACTS / "depth-report.md"

REQUIRED_STD = ["what", "why", "contracts", "failure modes", "how to verify"]
REQUIRED_RUNBOOK = ["symptoms", "metrics", "commands", "mitigations", "rollback"]
FORBIDDEN_TERMS = ["simple", "just", "obvious", "etc"]
SKIP_PREFIXES = (
    "_generated/",
    "_assets/",
    "_style/",
)
SKIP_FILES = set()


@dataclass
class Finding:
    path: str
    message: str


@dataclass
class DepthReport:
    findings: list[Finding] = field(default_factory=list)
    checked: int = 0

    def add(self, path: Path, message: str) -> None:
        self.findings.append(Finding(path=str(path.relative_to(ROOT)), message=message))


def rel(path: Path) -> str:
    return path.relative_to(DOCS).as_posix()


def should_skip(path: Path) -> bool:
    rp = rel(path)
    if rp in SKIP_FILES:
        return True
    return any(rp.startswith(prefix) for prefix in SKIP_PREFIXES)


def extract_headings(text: str) -> set[str]:
    headings = set()
    for line in text.splitlines():
        if line.startswith("## "):
            headings.add(line[3:].strip().lower())
    return headings


def count_code_fences(text: str) -> int:
    return text.count("```") // 2


def has_verify_command_block(text: str) -> bool:
    lowered = text.lower()
    idx = lowered.find("## how to verify")
    if idx == -1:
        return False
    tail = text[idx:]
    return "```" in tail and bool(re.search(r"\b(make|cargo|python3|scripts/)\b", tail))


def has_diagram(text: str) -> bool:
    if "```mermaid" in text:
        return True
    return bool(re.search(r"!\[[^\]]*\]\(([^)]+_assets/diagrams/[^)]+)\)", text))


def lint_file(path: Path, report: DepthReport) -> None:
    content = path.read_text(encoding="utf-8")
    headings = extract_headings(content)
    rp = rel(path)
    report.checked += 1

    is_index = path.name == "INDEX.md"
    is_runbook = rp.startswith("operations/runbooks/")
    is_major = rp.startswith(("reference/", "contracts/", "operations/")) and not is_index
    major_arch_docs = {
        "architecture/boundaries.md",
        "architecture/effects.md",
        "architecture/boundary-maps.md",
        "architecture/crate-boundary-dependency-graph.md",
    }
    is_arch = rp in major_arch_docs

    if is_runbook:
        for sec in REQUIRED_RUNBOOK:
            if sec not in headings:
                report.add(path, f"missing runbook section: {sec}")
    elif is_major:
        for sec in REQUIRED_STD:
            if sec not in headings:
                report.add(path, f"missing required section: {sec}")

    if is_major and count_code_fences(content) < 1:
        report.add(path, "requires at least 1 fenced code example")

    if is_major and not has_verify_command_block(content):
        report.add(path, "verify section must include runnable command block")

    lowered = content.lower()
    for term in FORBIDDEN_TERMS:
        if re.search(rf"\b{re.escape(term)}\b", lowered):
            report.add(path, f"contains forbidden handwavy term: {term}")

    if is_arch and not has_diagram(content):
        report.add(path, "architecture doc requires at least one diagram (mermaid or image)")


def write_report(report: DepthReport, threshold: int) -> None:
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Docs Depth Report",
        "",
        f"- Checked files: {report.checked}",
        f"- Findings: {len(report.findings)}",
        f"- Failure threshold: {threshold}",
        "",
    ]
    if report.findings:
        lines.append("## Findings")
        lines.append("")
        for finding in report.findings:
            lines.append(f"- `{finding.path}`: {finding.message}")
    else:
        lines.append("No findings.")
    REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    threshold = int(os.environ.get("DOCS_DEPTH_MAX_FINDINGS", "200"))
    report = DepthReport()
    for path in sorted(DOCS.rglob("*.md")):
        if should_skip(path):
            continue
        lint_file(path, report)
    write_report(report, threshold)
    if len(report.findings) > threshold:
        print(
            f"depth lint failed with {len(report.findings)} finding(s) "
            f"(threshold={threshold}); see {REPORT}"
        )
        return 1
    print(
        f"depth lint passed with {len(report.findings)} finding(s) "
        f"(threshold={threshold})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
