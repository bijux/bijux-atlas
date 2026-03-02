# Supply-Chain Policies

- Owner: `bijux-atlas-platform`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: describe the executable Docker publication policies that govern airgap readiness, registry promotion, and release provenance.

## Policy Sources

- `docker/airgap-policy.json`
- `docker/push-policy.json`
- `docker/policy.json`

## Airgap Build Policy

The airgap build policy requires locked base-image sources, digest-pinned inputs, and no ad-hoc network fetch tools in governed Docker builds.

## Registry Promotion Policy

The push policy defines the allowlisted registries, required signing posture, digest-only promotions, and provenance-bundle requirement before release publication.

## Verify

```bash
bijux dev atlas contracts docker
```

Expected result: Docker contracts validate the airgap, push, and provenance policies without drift.
