# Checks Reference

- Owner: `docs-governance`
- Type: `reference`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: provide one stable catalog of governed checks grouped by operational concern.

## Groups

### `rust`

| Check ID | Summary |
| --- | --- |
| `CHECK-RUSTFMT-001` | Verify Rust source formatting stays aligned with the pinned rustfmt policy. |
| `CHECK-RUST-CLIPPY-001` | Verify workspace lint cleanliness with warnings denied under the governed clippy policy. |

### `configs`

| Check ID | Summary |
| --- | --- |
| `CHECK-CONFIGS-LINT-001` | Verify configuration surfaces satisfy the fast config lint lane. |
| `CHECK-CONTRACTS-CONFIGS-REQUIRED-001` | Verify the required configuration contract-backed check suite remains green. |

### `docs`

| Check ID | Summary |
| --- | --- |
| `CHECK-DOCS-VALIDATE-001` | Verify documentation validation entrypoints remain green in the deterministic checks lane. |
| `CHECK-CONTRACTS-DOCS-REQUIRED-001` | Verify the required documentation contract-backed check suite remains green. |

### `ops`

| Check ID | Summary |
| --- | --- |
| `CHECK-K8S-VALIDATE-001` | Verify Kubernetes render and validation surfaces stay healthy in the deterministic checks lane. |

### `security`

| Check ID | Summary |
| --- | --- |
| `CHECK-SUPPLY-CHAIN-DENY-001` | Verify cargo-deny policy compliance for dependency and license policy controls. |
| `CHECK-SUPPLY-CHAIN-AUDIT-001` | Verify cargo-audit remains green for the dependency vulnerability audit lane. |

### `governance`

| Check ID | Summary |
| --- | --- |
| `CHECK-LINT-POLICY-001` | Verify governed root-domain lint checks stay green through the control plane. |
| `CHECK-SUITE-CI-FAST-001` | Verify the fast public control-plane check suite stays green for short feedback loops. |
| `CHECK-SUITE-CI-PR-001` | Verify the pull-request control-plane check suite remains green for merge blocking validation. |
| `CHECK-SUITE-CI-NIGHTLY-001` | Verify the nightly control-plane check suite remains green for broad regression coverage. |
| `CHECK-CONTRACTS-MAKE-REQUIRED-001` | Verify the required Make contract-backed check suite remains green. |
| `CHECK-CONTRACTS-REPO-REQUIRED-001` | Verify the required repository contract-backed check suite remains green. |

## See also

- [Check Reports](../reports/checks/index.md)
- [Reports Reference](../reports/index.md)
