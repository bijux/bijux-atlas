import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const BASE_RATE = Number(__ENV.BASE_RATE || 80);

export const options = {
  scenarios: {
    regional_spike: {
      executor: 'ramping-arrival-rate',
      startRate: BASE_RATE,
      timeUnit: '1s',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 96),
      maxVUs: Number(__ENV.MAX_VUS || 500),
      stages: [
        { target: BASE_RATE, duration: '20s' },
        { target: BASE_RATE * 10, duration: '10s' },
        { target: BASE_RATE * 10, duration: '60s' },
        { target: BASE_RATE, duration: '20s' }
      ]
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: [`p(99)<${Number(__ENV.P99_MS || 2000)}`]
  }
};

export default function () {
  const res = http.get(`${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-100000&limit=100`);
  check(res, { 'status is serviceable': (r) => r.status === 200 || r.status === 304 || r.status === 503 });
}
