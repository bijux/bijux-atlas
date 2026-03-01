# Docs Contract

## Scope

- Governed surface: `docs/` and `docs/contract.md`.
- Contributor entrypoint: `docs/README.md`.
- SSOT = bijux-dev-atlas contracts runner.
- Effects boundary: this group runs static contracts only.
- Tracked reader markdown stays under `docs/`.
- Generated docs evidence stays under `docs/_internal/generated/`.
- Runtime command output stays under `artifacts/run/<run_id>/`.
- Non-goals:
- This document does not replace executable contract checks.
- This document does not grant manual exception authority.

## Contract IDs

| ID | Title | Severity | Type(static/effect) | Enforced by | Artifacts |
| --- | --- | --- | --- | --- | --- |
| `DOC-001` | docs top-level sections stay curated | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-002` | docs root markdown stays on the curated surface | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-003` | docs paths stay within the depth budget | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-004` | docs directories stay within the sibling budget | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-005` | docs filenames avoid whitespace drift | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-006` | docs canonical entrypoint exists | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-007` | docs root files stay on the declared non-markdown surface | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-008` | docs top-level sections keep declared owners | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-009` | docs section manifest stays complete | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-010` | docs section entrypoints follow the declared manifest | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-011` | docs section index links resolve | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-012` | docs root entrypoint links resolve | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-013` | docs entrypoint pages declare owner metadata | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-014` | docs entrypoint page stability values stay normalized | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-015` | deprecated docs entrypoints name a replacement path | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-016` | docs section entrypoint owners align with the owner registry | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-017` | docs section manifest declares root entrypoint coverage | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-018` | docs root entrypoint links every declared root section | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-019` | docs entrypoint pages stay within the word budget | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-020` | stable docs entrypoint pages avoid placeholder markers | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-021` | docs entrypoint pages avoid raw tabs | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-022` | docs entrypoint pages avoid trailing whitespace | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-023` | docs entrypoint pages keep a single top-level heading | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-024` | docs entrypoint pages avoid absolute local file links | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-025` | docs entrypoint pages avoid raw http links | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-026` | docs root entrypoint avoids duplicate section index links | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-027` | docs section indexes link at least one local page | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-028` | docs section indexes avoid duplicate local page links | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-029` | docs root entrypoint pages avoid duplicate local page links | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-030` | docs index correctness report stays derivable | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-031` | docs root keeps a single canonical entrypoint | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-032` | docs broken links report is generated deterministically | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-033` | docs orphan pages report is generated deterministically | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-034` | docs metadata coverage report is generated deterministically | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-035` | docs duplication report is generated deterministically | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-036` | docs contract coverage report is generated deterministically | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-037` | spine pages require frontmatter | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-038` | spine page frontmatter values stay normalized | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-039` | stable spine pages keep complete metadata | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-040` | reference spine pages declare sources | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-041` | internal docs keep frontmatter boundary | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-042` | stable spine pages keep review dates normalized | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-043` | how-to spine pages declare verification | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-044` | docs frontmatter schema stays present | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-045` | reader utility pages keep shared metadata | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-046` | reader utility pages do not link into internal docs | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-047` | reader spine pages do not link into internal docs | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-048` | published docs titles stay unique | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-049` | published docs pages keep exactly one H1 | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-050` | operator golden paths do not reference internal docs | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-051` | docs home stays within the line budget | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-052` | docs keep a single start-here page | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-053` | docs keep a single glossary page | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-054` | mkdocs excludes drafts and internals from the reader build | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-055` | section indexes stay curated | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-056` | mkdocs nav starts with home and start here | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-057` | governance stays nested under development | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-058` | generated docs stay under the internal sink | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-059` | docs dashboard links required generated artifacts | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-060` | docs redirects target real pages | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-061` | docs registry validates against its schema | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-062` | docs sections validate against their schema | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |
| `DOC-063` | site map links the reader spine | `high` | `static` | `bijux dev atlas contracts docs` | `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json` |

## Enforcement mapping

| Contract | Command(s) |
| --- | --- |
| `DOC-001` | `bijux dev atlas contracts docs --mode static` |
| `DOC-002` | `bijux dev atlas contracts docs --mode static` |
| `DOC-003` | `bijux dev atlas contracts docs --mode static` |
| `DOC-004` | `bijux dev atlas contracts docs --mode static` |
| `DOC-005` | `bijux dev atlas contracts docs --mode static` |
| `DOC-006` | `bijux dev atlas contracts docs --mode static` |
| `DOC-007` | `bijux dev atlas contracts docs --mode static` |
| `DOC-008` | `bijux dev atlas contracts docs --mode static` |
| `DOC-009` | `bijux dev atlas contracts docs --mode static` |
| `DOC-010` | `bijux dev atlas contracts docs --mode static` |
| `DOC-011` | `bijux dev atlas contracts docs --mode static` |
| `DOC-012` | `bijux dev atlas contracts docs --mode static` |
| `DOC-013` | `bijux dev atlas contracts docs --mode static` |
| `DOC-014` | `bijux dev atlas contracts docs --mode static` |
| `DOC-015` | `bijux dev atlas contracts docs --mode static` |
| `DOC-016` | `bijux dev atlas contracts docs --mode static` |
| `DOC-017` | `bijux dev atlas contracts docs --mode static` |
| `DOC-018` | `bijux dev atlas contracts docs --mode static` |
| `DOC-019` | `bijux dev atlas contracts docs --mode static` |
| `DOC-020` | `bijux dev atlas contracts docs --mode static` |
| `DOC-021` | `bijux dev atlas contracts docs --mode static` |
| `DOC-022` | `bijux dev atlas contracts docs --mode static` |
| `DOC-023` | `bijux dev atlas contracts docs --mode static` |
| `DOC-024` | `bijux dev atlas contracts docs --mode static` |
| `DOC-025` | `bijux dev atlas contracts docs --mode static` |
| `DOC-026` | `bijux dev atlas contracts docs --mode static` |
| `DOC-027` | `bijux dev atlas contracts docs --mode static` |
| `DOC-028` | `bijux dev atlas contracts docs --mode static` |
| `DOC-029` | `bijux dev atlas contracts docs --mode static` |
| `DOC-030` | `bijux dev atlas contracts docs --mode static` |
| `DOC-031` | `bijux dev atlas contracts docs --mode static` |
| `DOC-032` | `bijux dev atlas contracts docs --mode static` |
| `DOC-033` | `bijux dev atlas contracts docs --mode static` |
| `DOC-034` | `bijux dev atlas contracts docs --mode static` |
| `DOC-035` | `bijux dev atlas contracts docs --mode static` |
| `DOC-036` | `bijux dev atlas contracts docs --mode static` |
| `DOC-037` | `bijux dev atlas contracts docs --mode static` |
| `DOC-038` | `bijux dev atlas contracts docs --mode static` |
| `DOC-039` | `bijux dev atlas contracts docs --mode static` |
| `DOC-040` | `bijux dev atlas contracts docs --mode static` |
| `DOC-041` | `bijux dev atlas contracts docs --mode static` |
| `DOC-042` | `bijux dev atlas contracts docs --mode static` |
| `DOC-043` | `bijux dev atlas contracts docs --mode static` |
| `DOC-044` | `bijux dev atlas contracts docs --mode static` |
| `DOC-045` | `bijux dev atlas contracts docs --mode static` |
| `DOC-046` | `bijux dev atlas contracts docs --mode static` |
| `DOC-047` | `bijux dev atlas contracts docs --mode static` |
| `DOC-048` | `bijux dev atlas contracts docs --mode static` |
| `DOC-049` | `bijux dev atlas contracts docs --mode static` |
| `DOC-050` | `bijux dev atlas contracts docs --mode static` |
| `DOC-051` | `bijux dev atlas contracts docs --mode static` |
| `DOC-052` | `bijux dev atlas contracts docs --mode static` |
| `DOC-053` | `bijux dev atlas contracts docs --mode static` |
| `DOC-054` | `bijux dev atlas contracts docs --mode static` |
| `DOC-055` | `bijux dev atlas contracts docs --mode static` |
| `DOC-056` | `bijux dev atlas contracts docs --mode static` |
| `DOC-057` | `bijux dev atlas contracts docs --mode static` |
| `DOC-058` | `bijux dev atlas contracts docs --mode static` |
| `DOC-059` | `bijux dev atlas contracts docs --mode static` |
| `DOC-060` | `bijux dev atlas contracts docs --mode static` |
| `DOC-061` | `bijux dev atlas contracts docs --mode static` |
| `DOC-062` | `bijux dev atlas contracts docs --mode static` |
| `DOC-063` | `bijux dev atlas contracts docs --mode static` |

## Output artifacts

- `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.json`
- `artifacts/run/<run_id>/gates/contracts/docs/<profile>/<mode>/docs.inventory.json`

## Contract to Gate mapping

- Gate: `contracts::docs`
- Aggregate gate: `contracts::all`

## Exceptions policy

- No exceptions are allowed by this document.
