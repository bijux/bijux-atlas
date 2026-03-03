# Why CI Is A Thin Shell

- Owner: `bijux-atlas-governance`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff8cd5f299e568c93feec8b4d40347bf1c5a93c4`
- Reason to exist: define the permanent rule that CI orchestrates atlas and does not become a second policy engine.

## Rule

- CI may set up toolchains, restore caches, invoke `bijux-dev-atlas` or thin `make` delegates, and upload artifacts.
- CI may not introduce new workflow-local validation logic unless it is registered in `configs/ci/policy-outside-control-plane.json` with an atlas replacement plan.
- Temporary exceptions must be recorded in `configs/ci/workflow-allowlist.json` with owner, expiry, and renewal reference.

## Allowed Patterns

- `uses: actions/checkout@...`
- `uses: dtolnay/rust-toolchain@...`
- `uses: actions/cache/restore@...`
- `uses: actions/cache/save@...`
- `uses: actions/upload-artifact@...`
- `run: cargo run --locked -q -p bijux-dev-atlas -- ...`
- `run: make <thin-target>`
- `run: mkdir -p "artifacts/${RUN_ID}"` and closely related artifact-root setup
- `run: python3 -m pip install ...` for toolchain setup only

## Review Rule

- If a workflow step contains validation logic that depends on `grep`, `jq`, `awk`, `sed`, `find`, inline Python, or inline Node, move that logic into atlas.
- If a step remains mixed or lane-sized, it must be covered by the workflow allowlist and the policy registry until the atlas surface is complete.

## Verify Success

- `cargo run -q -p bijux-dev-atlas -- ci report --kind workflow-lint --format json`
- `cargo run -q -p bijux-dev-atlas -- ci verify workflow-policy --format json`

## What To Read Next

- [CI overview](ci-overview.md)
- [Control-plane](../control-plane/index.md)
- [Reports reference](../reference/reports/index.md)
