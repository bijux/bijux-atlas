"""Resolve a committed consumer baseline from exact accepted GitHub source."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

from scripts.project_bijux_docs import published_bytes, source_context

PIN = '.github/standards/bijux-std.sha'
BASELINE = 'bijux-docs/config/mkdocs-baseline.json'
SHA = re.compile(r'[a-f0-9]{40}\Z')


def git(root: Path, *arguments: str) -> str:
    result = subprocess.run(['git', '-C', str(root), *arguments], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError('Prior asset authority: required Git source is unavailable: ' + ' '.join(arguments))
    return result.stdout.strip()


def authority(context: dict) -> Path:
    root = Path(context['authority']).resolve()
    if (git(root, 'rev-parse', '--show-toplevel') != str(root)
            or git(root, 'remote', 'get-url', 'origin') != context['origin']
            or git(root, 'rev-parse', 'HEAD') != context['sha']
            or git(root, 'status', '--porcelain', '--untracked-files=all')):
        raise RuntimeError('Prior asset authority: exact clean accepted GitHub checkout is required')
    return root


def prior_javascript(repository: Path, shared: Path) -> tuple[str, ...] | None:
    """Never infer prior ownership from local overrides or untracked receipts.

    The normal/current list path does not call this function. The caller supplies
    no predecessor override: the consumer's committed pin selects the prior tree.
    """
    if os.environ.get('BIJUX_DOCS_SOURCE_MODE') != 'accepted-github':
        return None
    context = source_context()
    root = repository.resolve()
    if git(root, 'rev-parse', '--show-toplevel') != str(root):
        raise RuntimeError('Prior asset authority: explicit consumer repository root is required')
    pin = root / PIN
    if any(path.is_symlink() for path in (pin, pin.parent, pin.parent.parent)) or not pin.is_file():
        raise RuntimeError('Prior asset authority: regular working standard pin is required')
    if pin.read_text().strip() != context['sha']:
        raise RuntimeError('Prior asset authority: working standard pin differs from accepted source')
    committed = git(root, 'ls-tree', 'HEAD', '--', PIN)
    if not re.fullmatch(r'100644 blob [a-f0-9]{40}\t' + re.escape(PIN), committed):
        raise RuntimeError('Prior asset authority: committed regular standard pin is required')
    previous = git(root, 'show', 'HEAD:' + PIN)
    if not SHA.fullmatch(previous):
        raise RuntimeError('Prior asset authority: committed exact previous standard pin is required')
    if previous == context['sha']:
        return None
    authority(context)
    current = published_bytes(context, context['sha'], BASELINE)
    if hashlib.sha256((shared / 'config/mkdocs-baseline.json').read_bytes()).digest() != hashlib.sha256(current).digest():
        raise RuntimeError('Prior asset authority: current baseline differs from accepted Git source')
    data = published_bytes(context, previous, BASELINE)
    authority(context)
    try:
        baseline = json.loads(data)
        names = baseline['extra_javascript']
        if (not isinstance(names, list) or not names
                or not all(isinstance(name, str) and name and not any(ord(char) < 32 for char in name) for name in names)
                or len(names) != len(set(names))):
            raise ValueError('invalid script list')
    except (ValueError, KeyError, TypeError) as error:
        raise RuntimeError('Prior asset authority: invalid previous accepted script baseline') from error
    return tuple(names)
