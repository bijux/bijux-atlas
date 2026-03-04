# Artifact Integrity Verification

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define integrity checks for dataset artifacts and manifests.

## Integrity Controls

- verify artifact SHA-256 against manifest checksum
- verify manifest checksum against canonical manifest payload
- verify artifact signature for signed artifacts

## Runtime Outcomes

- integrity failures are denied and never served
- corrupted artifacts are quarantined for safety
- integrity and tamper counters are incremented for observability
