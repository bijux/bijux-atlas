# Profile Intent

`ops/k8s/charts/bijux-atlas/values.yaml` is the chart default surface. The files in this
directory are overlays that describe named operating intents: fast CI validation, local kind
installs, offline startup, load testing, and production rollouts. `profile-baseline.yaml` records
the common operator defaults so each profile file can stay focused on the deltas that matter in its
target environment, while `profiles.json` is the stable registry for purpose, risk, ownership, and
required controls.
