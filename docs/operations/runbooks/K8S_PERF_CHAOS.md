# Kubernetes Performance Chaos Runbook

Scenarios covered:
- Pod kill under load (kill -9 equivalent through forced delete).
- Noisy-neighbor CPU throttling.
- Memory limit pressure.
- Regional traffic spike (10x for 60s).

## 1) Pod Kill Under Load

1. Start load:
```bash
BASE_URL=http://atlas.default.svc.cluster.local:8080 k6 run load/k6/suites/warm_steady.js
```
2. Force-delete one pod during steady load:
```bash
kubectl delete pod -n <ns> -l app.kubernetes.io/name=bijux-atlas --force --grace-period=0
```
3. Verify bounded tail:
- p99 from load output remains below agreed threshold.
- `/metrics` shows no sustained surge in `bijux_http_requests_total{status="5xx"}`.

## 2) Noisy Neighbor CPU Throttle

1. Set low CPU limit on one replica (test namespace only).
2. Run:
```bash
BASE_URL=http://atlas.default.svc.cluster.local:8080 k6 run load/k6/suites/noisy_neighbor_cpu_throttle.js
```
3. Validate:
- Cheap requests remain available.
- Heavy requests degrade with controlled `503` instead of broad failures.
- `bijux_overload_shedding_active` reflects active shedding.

## 3) Memory Pressure

1. Lower memory limit temporarily (test env).
2. Run mixed + soak:
```bash
BASE_URL=http://atlas.default.svc.cluster.local:8080 k6 run load/k6/suites/mixed_80_20.js
BASE_URL=http://atlas.default.svc.cluster.local:8080 k6 run load/k6/suites/soak_30m.js
```
3. Validate:
- No cascading restart storm.
- Pod restarts do not cause global 5xx spikes.
- Cache and shard caps limit resource pressure.

## 4) Regional Traffic Spike

```bash
BASE_URL=http://atlas.default.svc.cluster.local:8080 k6 run load/k6/suites/regional_spike_10x_60s.js
```

Validate:
- HPA reacts using custom metrics.
- Shedding activates for heavy class when needed.
- Cheap endpoints remain healthy.
