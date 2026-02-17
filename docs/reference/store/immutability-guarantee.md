# Dataset Immutability Guarantee

Published datasets are immutable.

Rules:

- A published dataset path (`release/species/assembly`) is read-only.
- Existing manifests and SQLite artifacts must never be overwritten in place.
- Corrections require a new publish identity (new release or explicit corrective release tag).
- API serving layer must treat published artifacts as immutable snapshots.
