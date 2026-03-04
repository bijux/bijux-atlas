# Metric Types

- Counter: monotonic event totals (requests, errors, rejects).
- Gauge: current state (queue depth, memory, thread usage).
- Histogram: latency and size distributions (request duration, response bytes).

Selection rule:

- Use counters for rates and SLO burn math.
- Use gauges for saturation and capacity.
- Use histograms for latency percentiles and payload spread.
