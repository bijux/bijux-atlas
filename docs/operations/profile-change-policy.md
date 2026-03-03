# Profile Change Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define what counts as a breaking profile change.

## Breaking Changes

Treat any of the following as a breaking profile change:

- Changing `intendedUse`, risk level, or owner in `ops/k8s/values/profiles.json`.
- Changing a profile’s `forbiddenToggles` or `requiredToggles`.
- Changing required network policy posture or HPA posture.
- Changing image immutability expectations for production-oriented profiles.
- Changing a profile from ephemeral storage to persistent storage, or the reverse.

## Non-Breaking Changes

The following are non-breaking when the declared profile intent stays intact:

- Tightening probes without changing the profile’s intent.
- Raising resource sizes inside the same expected resource class.
- Adding documentation that clarifies existing constraints.

## Verify

When changing a profile, update the matching `ops/k8s/values/*.yaml`, `ops/k8s/values/profiles.json`,
and `ops/k8s/rollout-safety-contract.json` together.
