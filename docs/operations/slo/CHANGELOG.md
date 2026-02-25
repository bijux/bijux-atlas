# SLO Changelog

## 2026-02-20

- Pre-release cleanup: legacy compatibility paths removed across docs/ops/config/runtime.
- Runtime env contract enforcement added; unknown `ATLAS_*`/`BIJUX_*` vars now fail by default.

## 2026-02-18

- Introduced SLO v1 SSOT under `configs/ops/slo/`.
- Defined endpoint classes (`cheap`, `standard`, `heavy`).
