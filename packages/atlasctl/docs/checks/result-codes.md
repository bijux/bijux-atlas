# Check Result Codes

Every check emits a stable machine-readable `result_code`.

## Contract

- format: `UPPER_SNAKE_CASE`
- deterministic for the same policy violation class
- included in json report rows

## Purpose

- machine routing for automation
- stable taxonomy linkage to higher-level error/report systems
- avoids parsing human message strings

## Guidance

- prefer intent-specific codes over generic fallback codes
- preserve existing codes for backward compatibility where external tooling depends on them
- use remediation text and docs links for human guidance; keep `result_code` concise and stable
