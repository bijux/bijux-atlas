import http from 'k6/http';
import { check } from 'k6';
import { Counter } from 'k6/metrics';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const BASE_RATE = Number(__ENV.BASE_RATE || 80);
const RSS_CAP_BYTES = Number(__ENV.RSS_CAP_BYTES || 2147483648); // 2Gi default

const cheapSurvival = new Counter('cheap_survival_ok_total');
const heavyShed = new Counter('heavy_shed_observed_total');
const overloadActive = new Counter('overload_active_observed_total');
const queueMetricObserved = new Counter('queue_depth_metric_observed_total');
const queuePositiveObserved = new Counter('queue_depth_positive_observed_total');
const rssCapExceeded = new Counter('rss_cap_exceeded_total');

export const options = {
  scenarios: {
    spike_proof: {
      executor: 'ramping-arrival-rate',
      startRate: BASE_RATE,
      timeUnit: '1s',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 96),
      maxVUs: Number(__ENV.MAX_VUS || 500),
      stages: [
        { target: BASE_RATE, duration: '20s' },
        { target: BASE_RATE * 10, duration: '15s' },
        { target: BASE_RATE * 10, duration: '60s' },
        { target: BASE_RATE, duration: '20s' }
      ]
    }
  },
  thresholds: {
    http_req_duration: [`p(99)<${Number(__ENV.P99_MS || 2500)}`],
    cheap_survival_ok_total: ['count>100'],
    heavy_shed_observed_total: ['count>0'],
    overload_active_observed_total: ['count>0'],
    queue_depth_metric_observed_total: ['count>0'],
    queue_depth_positive_observed_total: ['count>0'],
    rss_cap_exceeded_total: ['count==0']
  }
};

function parseMetricLine(metrics, name) {
  const line = metrics
    .split('\n')
    .find((l) => l.startsWith(name + '{') || l.startsWith(name + ' '));
  if (!line) return null;
  const parts = line.trim().split(/\s+/);
  if (parts.length < 2) return null;
  const n = Number(parts[parts.length - 1]);
  return Number.isFinite(n) ? n : null;
}

export default function () {
  const cheap = http.get(`${BASE}/v1/version`);
  if (check(cheap, { 'cheap endpoint survives spike': (r) => r.status === 200 })) {
    cheapSurvival.add(1);
  }

  const heavy = http.get(`${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-250000&limit=100`);
  check(heavy, {
    'heavy path serviceable': (r) => [200, 304, 422, 429, 503].includes(r.status),
  });
  if (heavy.status === 429 || heavy.status === 503) {
    heavyShed.add(1);
  }

  const overload = http.get(`${BASE}/healthz/overload`);
  if (overload.status === 503) {
    overloadActive.add(1);
  }
  if (overload.status === 200) {
    try {
      const o = overload.json();
      if (o && o.overloaded === true) {
        overloadActive.add(1);
      }
    } catch (_) {
      // ignore non-json noise
    }
  }

  if (__ITER % 10 === 0) {
    const metrics = http.get(`${BASE}/metrics`);
    if (metrics.status === 200) {
      const depth = parseMetricLine(metrics.body, 'bijux_request_queue_depth');
      if (depth !== null) {
        queueMetricObserved.add(1);
        if (depth > 0) {
          queuePositiveObserved.add(1);
        }
      }
      const rss = parseMetricLine(metrics.body, 'process_resident_memory_bytes');
      if (rss !== null && rss > RSS_CAP_BYTES) {
        rssCapExceeded.add(1);
      }
    }
  }
}
