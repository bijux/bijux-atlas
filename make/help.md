# Make Public Targets

The public surface is defined by `CURATED_TARGETS` in `make/makefiles/root.mk` and printed by `make help`.

- Public targets must have one-line descriptions.
- Internal targets must use `_internal-` prefix.
- Public targets delegate to `bijux dev atlas ...` or `cargo ...`.
- Public targets write output under `artifacts/`.

## Public target list

- help: print the bounded public make surface.
- doctor: run control-plane and environment doctor checks.
- fmt: run workspace formatting checks.
- lint: run strict workspace lint checks.
- test: run workspace automated test suites.
- build: build workspace binaries.
- docker: alias of `docker-validate`.
- docker-validate: run docker validation gates.
- docker-build: run docker build gate.
- docker-smoke: run docker smoke gate.
- docker-sbom: run docker sbom gate.
- docker-scan: run docker vulnerability scan gate.
- docker-lock: run docker digest lock gate.
- docker-release: run docker release gate (explicit override).
- docker-gate: run complete docker gate sequence.
- k8s-render: render Kubernetes manifests deterministically.
- k8s-validate: run Kubernetes manifest validation and policy checks.
- stack-up: stand up local ops stack.
- stack-down: tear down local ops stack.
- ops-fast: run fast CI suite checks.
- ops-pr: run pull-request CI suite checks.
- ops-nightly: run nightly CI suite checks.
- make-target-list: regenerate `make/target-list.json` from curated targets.
