# Run Ops Locally

Local ops runs should use atlasctl command surfaces only.

## Prereqs

1. Run `./bin/atlasctl ops prereqs --report text`
2. Run `./bin/atlasctl ops doctor --report text`

## Common Flows

1. Stack up: `./bin/atlasctl ops stack up --report text`
2. Deploy: `ATLASCTL_OPS_DEPLOY_ALLOW_APPLY=1 ./bin/atlasctl ops deploy apply --report text`
3. Smoke lane: `./bin/atlasctl ops smoke --report text`
4. Tear down: `./bin/atlasctl ops stack down --report text`

## Evidence

- Evidence root: `artifacts/evidence/<area>/<run_id>/...`
- Suite output: `artifacts/isolate/<run_id>/atlasctl-suite/results.json`

