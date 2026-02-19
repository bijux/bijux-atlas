# Make Targets

Generated from `makefiles/targets.json`. Do not edit manually.

| target | description | owner | area | lane |
|---|---|---|---|---|
| `cargo/all` | Local exhaustive Rust lane | `rust-platform` | `cargo` | `dev` |
| `cargo/audit` | Cargo audit gate | `rust-platform` | `cargo` | `dev` |
| `cargo/bench-smoke` | Cargo benchmark smoke lane | `rust-platform` | `cargo` | `dev` |
| `cargo/coverage` | Cargo coverage lane (kept out of root) | `rust-platform` | `cargo` | `dev` |
| `cargo/fmt` | Cargo fmt gate | `rust-platform` | `cargo` | `dev` |
| `cargo/lint` | Cargo lint gate | `rust-platform` | `cargo` | `dev` |
| `cargo/test` | Cargo CI-profile tests | `rust-platform` | `cargo` | `dev` |
| `cargo/test-all` | Cargo full nextest tests | `rust-platform` | `cargo` | `dev` |
| `cargo/test-contracts` | Cargo contract-focused tests | `rust-platform` | `cargo` | `dev` |
| `cargo/test-fast` | Cargo fast unit-focused tests | `rust-platform` | `cargo` | `dev` |
| `ci/all` | Deterministic CI superset | `build-and-release` | `policies` | `ci` |
| `clean-all` | Clean all allowed generated dirs (CONFIRM=YES) | `repo-hygiene` | `scripts` | `dev` |
| `clean-safe` | Clean only safe generated make artifact dirs | `repo-hygiene` | `scripts` | `dev` |
| `configs/all` | Configs schema and drift lane | `config-governance` | `configs` | `dev` |
| `configs/check` | Configs schema+drift checks | `config-governance` | `configs` | `dev` |
| `docs/all` | Docs lane | `docs-governance` | `docs` | `dev` |
| `docs/build` | Docs build artifacts | `docs-governance` | `docs` | `dev` |
| `docs/check` | Docs fast verification | `docs-governance` | `docs` | `dev` |
| `doctor` | Print tool/env/path diagnostics and store report | `build-and-release` | `policies` | `dev` |
| `explain` | Explain one public target, lanes, and internal expansion | `build-and-release` | `policies` | `dev` |
| `gates` | Show public targets grouped by area | `build-and-release` | `policies` | `dev` |
| `graph` | Print a compact dependency graph for a public target | `build-and-release` | `policies` | `dev` |
| `help` | Show curated public make targets grouped by namespace | `build-and-release` | `policies` | `dev` |
| `lane-cargo` | Lane: rust checks/tests with isolated artifacts | `rust-platform` | `cargo` | `dev` |
| `lane-configs-policies` | Lane: configs and policies checks | `config-governance` | `configs` | `dev` |
| `lane-docs` | Lane: docs build/freeze/hardening | `docs-governance` | `docs` | `dev` |
| `lane-ops` | Lane: ops lint/contracts without cluster bring-up | `ops-platform` | `ops` | `dev` |
| `lane-scripts` | Lane: scripts lint/tests/audit | `repo-hygiene` | `scripts` | `dev` |
| `list` | List curated public targets with one-line descriptions | `build-and-release` | `policies` | `dev` |
| `local/all` | Run everything meaningful locally | `build-and-release` | `policies` | `dev` |
| `nightly/all` | Slow nightly suites (perf/load/drills/realdata) | `build-and-release` | `policies` | `ci` |
| `ops/all` | Ops lint/schemas/contracts lane | `ops-platform` | `ops` | `dev` |
| `ops/check` | Ops fast verification | `ops-platform` | `ops` | `dev` |
| `ops/smoke` | Ops smoke checks | `ops-platform` | `ops` | `dev` |
| `ops/suite` | Ops suite checks | `ops-platform` | `ops` | `dev` |
| `policies/all` | Policies lane (deny/audit/policy checks) | `policy-governance` | `policies` | `dev` |
| `policies/check` | Policies deny/audit/relaxation checks | `policy-governance` | `policies` | `dev` |
| `prereqs` | Validate required binaries and versions and store report | `build-and-release` | `policies` | `dev` |
| `print-env` | Print key environment variables used by lanes | `build-and-release` | `policies` | `dev` |
| `quick` | Minimal tight loop (fmt + lint + test) | `build-and-release` | `cargo` | `dev` |
| `report/junit` | Generate optional JUnit report | `build-and-release` | `policies` | `ci` |
| `report/md` | Generate markdown summary for PR comments | `build-and-release` | `docs` | `dev` |
| `report/merge` | Merge lane reports into unified JSON | `build-and-release` | `policies` | `dev` |
| `report/print` | Print human make lane summary | `build-and-release` | `policies` | `dev` |
| `repro` | Re-run one lane deterministically with optional seed | `build-and-release` | `policies` | `dev` |
| `retry` | Retry target with same RUN_ID | `build-and-release` | `policies` | `dev` |
| `root` | CI-fast lane subset (no cluster bring-up) | `build-and-release` | `policies` | `ci` |
| `root-local` | All lanes in parallel plus ops smoke lane | `build-and-release` | `policies` | `dev` |
| `root-local-open` | Open or print the root-local summary report | `build-and-release` | `policies` | `dev` |
| `root-local-summary` | Print pass/fail summary by lane | `build-and-release` | `policies` | `dev` |
| `scripts/all` | Scripts lint/tests/audit lane | `repo-hygiene` | `scripts` | `dev` |
| `scripts/check` | Deterministic scripts checks | `repo-hygiene` | `scripts` | `dev` |
