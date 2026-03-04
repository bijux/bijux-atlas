# Metric Retention Policy

- Hot retention: high-resolution data for active incident windows.
- Warm retention: downsampled data for trend and release comparison.
- Long retention: SLO and governance evidence slices.

Operational rule:

- Keep retention tier settings aligned with storage budget and alert lookback windows.
- Do not reduce retention below SLO burn analysis requirements.
