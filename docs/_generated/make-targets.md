# Make Targets

Generated from `makefiles/targets.json`. Do not edit manually.

| target | description | owner | area | lane |
|---|---|---|---|---|
| `audit` | Run dependency and policy audits | `rust-platform` | `cargo` | `dev` |
| `chart` | Run chart verification checks | `ops-platform` | `ops` | `dev` |
| `ci` | Run deterministic CI superset | `build-and-release` | `policies` | `ci` |
| `docker` | Run docker contract checks | `ops-platform` | `ops` | `dev` |
| `docs` | Run docs verification lane | `docs-governance` | `docs` | `dev` |
| `doctor` | Print tool and environment diagnostics | `build-and-release` | `ops` | `dev` |
| `fmt` | Run formatter checks | `rust-platform` | `cargo` | `dev` |
| `lint` | Run lint checks | `rust-platform` | `cargo` | `dev` |
| `nightly` | Run slow nightly suites | `build-and-release` | `policies` | `ci` |
| `ops` | Run ops verification lane | `ops-platform` | `ops` | `dev` |
| `root` | Run CI-fast lane set and print lane summary | `build-and-release` | `policies` | `ci` |
| `root-local` | Run local full lane set and print lane summary | `build-and-release` | `policies` | `dev` |
| `test` | Run test suite | `rust-platform` | `cargo` | `dev` |
