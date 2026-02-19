#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

python3 - <<'PY'
from pathlib import Path
import re
import sys

errors=[]
allowed_non_kebab = {
    'docs/operations/slo/BASELINE_UPDATE_POLICY.md',
    'docs/operations/slo/CHANGELOG.md',
    'docs/operations/slo/CHANGE_POLICY.md',
    'docs/operations/slo/GLOSSARY.md',
    'docs/operations/slo/NON_GOALS.md',
    'docs/operations/slo/RELEASE_GATE.md',
    'docs/operations/slo/SCOPE.md',
    'docs/operations/slo/SLA_DECISION_ADR_TEMPLATE.md',
    'docs/operations/slo/SLA_EXCLUSIONS_PLANNED.md',
    'docs/operations/slo/SLA_POLICY.md',
    'docs/operations/slo/SLIS.md',
    'docs/operations/slo/SLOS.md',
    'docs/operations/slo/WHAT_WE_DONT_MEASURE_YET.md',
    'docs/operations/slo/WHY_THESE_SLIS.md',
}
for p in sorted(Path('docs').rglob('*.md')):
    name=p.name
    rel=str(p)
    if rel in allowed_non_kebab:
        continue
    if rel=='docs/contracts/README.md':
        continue
    if name=='README.md':
        errors.append(f'forbidden docs README: {rel}')
        continue
    if name=='INDEX.md':
        continue
    if rel.startswith('docs/_generated/'):
        continue
    if name=='DEPTH_POLICY.md':
        continue
    if name=='CONCEPT_REGISTRY.md':
        continue
    if name=='DEPTH_RUBRIC.md':
        continue
    if name=='STYLE.md':
        continue
    if re.match(r'ADR-\d{4}-[a-z0-9-]+\.md$', name):
        continue
    if not re.match(r'^[a-z0-9]+(?:-[a-z0-9]+)*\.md$', name):
        errors.append(f'non-kebab docs filename: {rel}')
    if re.search(r'(notes|misc|human_machine)', name, re.I):
        errors.append(f'forbidden filename pattern: {rel}')

if errors:
    print('doc naming check failed:', file=sys.stderr)
    for e in errors:
        print(f'- {e}', file=sys.stderr)
    raise SystemExit(1)
print('doc naming check passed')
PY
