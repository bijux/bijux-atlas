# Script Naming Convention

- Owner: `platform`

## Rules

- Shell scripts: kebab-case, verb-noun (`check-layout.sh`, `generate-report.sh`).
- Python scripts: snake_case (`check_layout.py`, `generate_report.py`).
- Public wrappers should mirror the canonical script name they delegate to.
- Avoid overloaded scripts; split orchestration (public) from logic (internal/tools).

## Why

Consistent names make scripts discoverable and reduce accidental duplication.

## Verify

```bash
make scripts-audit
```
