# Pin Promotion Workflow

- Owner: bijux-atlas-operations
- Stability: stable

## Flow

1. Edit candidate values in `ops/inventory/pins.yaml`.
2. Run `bijux dev atlas ops doctor --format json`.
3. Run `bijux dev atlas ops pins update --format json --allow-write`.
4. Review inventory drift and confirm image/dataset coverage.
5. Commit pin updates and generated index evidence together.
6. Promote release with immutable pins for that release line.

## Guardrails

- `ops/inventory/pins.yaml` is the only SSOT for image and dataset pins.
- `latest` tags are forbidden.
- sha256 digest format is enforced when digest form is used.
- Semver format is required for `versions` entries.
