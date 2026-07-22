---
title: Concurrency Stress
audience: operators
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Concurrency and Saturation

Concurrency testing locates the point where clients begin competing for Atlas
admission, CPU, store access, cache space, and database work. The useful result
is a repeatable operating envelope and controlled overload behavior, not a peak
request count.

## Declared Scenario Shapes

`ops/load/generated/concurrency-stress-scenarios.json` catalogs three shapes:

| Scenario | Workload | Concurrency role |
| --- | --- | --- |
| `load-single-client-baseline` | query | low-contention reference |
| `load-multi-client-concurrency` | mixed | shared-resource contention |
| `load-saturation-stress` | mixed | pressure at or beyond intended limits |

These records contain no target rate, duration, client count, query mix,
dataset, resource profile, or threshold. They are catalog entries, not
executable load suites. None is present in `ops/load/load.toml`, so `ops load
run` cannot execute these IDs directly.

Do not treat the generated file as performance evidence. A governed experiment
must add the missing parameters and preserve the actual harness result.

## Build the Saturation Curve

```mermaid
flowchart LR
    Base["single-client baseline"] --> Clients["increase client concurrency"]
    Clients --> Rate["increase offered rate"]
    Rate --> Observe["measure service and resources"]
    Observe --> Boundary{"contention boundary stable?"}
    Boundary -- no --> Clients
    Boundary -- yes --> Overload["cross boundary deliberately"]
    Overload --> Recover["remove pressure and prove recovery"]
```

Change one pressure dimension at a time. Fix the release, dataset, query
corpus, cache state, resources, and dependency versions. Report offered rate
and completed throughput separately; queues can make them diverge sharply.

For each point, record:

- clients, arrival model, target rate, duration, and request-class mix;
- p50, p95, and p99 latency plus status and error classes;
- completed throughput, in-flight work, queues, and overload state;
- CPU use and throttling, memory, cache growth, store latency, and connections;
- replica count, HPA actions, and workload distribution;
- query correctness and response-size bounds.

## Experiment Ledger

Keep the demand placed on the system distinct from work the system completed.
The ledger is the join key between the load generator, Atlas telemetry, and the
recovery observation.

| Record | Minimum fields |
| --- | --- |
| Candidate | revision, artifact, configuration, dataset, dependencies |
| Generator | image, script, resources, location, clock source |
| Offer | arrival model, clients, requested rate, duration, traffic mix |
| Service | admitted, completed, rejected, failed, latency, correctness |
| Resources | replicas, CPU, memory, queues, connections, cache, store |
| Recovery | pressure removal, queue drain, convergence, probe restoration |

```mermaid
flowchart LR
    Offer["offered work"] --> Admission{"admission decision"}
    Admission -- admitted --> Terminal{"terminal outcome"}
    Admission -- rejected --> Controlled["controlled overload"]
    Terminal -- correct_completion --> Capacity["completed throughput"]
    Terminal -- error_or_timeout --> Instability["service instability"]
    Capacity --> Recovery["post-load recovery"]
    Controlled --> Recovery
    Instability --> Recovery
```

Throughput means completed, contract-correct work. Offered requests, accepted
connections, and queued operations are demand indicators; none may substitute
for completed throughput.

Reject the measurement when the generator saturates before Atlas, clocks or
measurement windows cannot be reconciled, autoscaling changes outside the
declared envelope, or repetitions use different warm-up, cache state, dataset,
or query mix. Those runs may diagnose the harness, but they cannot establish a
capacity boundary.

## Attribute the First Bottleneck

At the first unstable point, identify the resource that saturated before
changing capacity. CPU throttling, memory pressure, queue depth, store latency,
connection pools, cache misses, and load-generator limits can produce similar
latency curves but require different actions.

Repeat the boundary after changing only the suspected constraint. If the knee
does not move as predicted, keep the diagnosis open. Scaling replicas without
checking store and cache amplification can increase failure rather than
capacity.

## Protected Behavior Under Overload

Atlas separates cheap and heavy query admission. Saturation evidence should
show that heavy work is rejected with the declared policy response while cheap
health, readiness, version, and catalog paths retain their contract. Observe
the response code and error envelope; timeouts alone are uncontrolled failure.

After load stops, verify that queues drain, overload state clears, memory and
cache settle inside expected bounds, and normal requests recover. A service
that meets peak thresholds but does not recover has failed the experiment.

## Make a Capacity Claim

Report three regions: normal operation, onset of contention, and controlled
overload. Use the lowest repeatable boundary across valid repetitions. Bind the
claim to its latency, error, resource, traffic-mix, and correctness conditions.

Reject a result when parameters are missing, correctness changes, protected
paths collapse, telemetry cannot identify the bottleneck, or repetitions move
the boundary materially without explanation.

Use [Baseline management](baseline-management.md) for comparison custody and
[Thresholds and budgets](thresholds-and-budgets.md) for approval semantics.
