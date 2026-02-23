# Reproduce A Release Locally

Use atlasctl product commands only.

## Prereqs

1. `./bin/atlasctl ops pins check --report text`
2. `docker`, `helm`, `git` installed and matching pinned versions where applicable

## Build + Validate

1. `IMAGE_VERSION=$(git rev-parse --short=12 HEAD) ./bin/atlasctl product build`
2. `./bin/atlasctl product validate`
3. `./bin/atlasctl product inventory`

## Compare Two Builds

1. `./bin/atlasctl product diff <old-artifact-manifest.json> <new-artifact-manifest.json>`

## Publish (Internal)

1. `CI=1 ./bin/atlasctl product docker release`
2. `./bin/atlasctl product publish --internal`

