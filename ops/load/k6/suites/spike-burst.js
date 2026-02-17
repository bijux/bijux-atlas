import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const P95 = Number(__ENV.P95_MS || 1200);

export const options = {
  scenarios: {
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 50,
      timeUnit: '1s',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 80),
      maxVUs: Number(__ENV.MAX_VUS || 500),
      stages: [
        { target: 50, duration: '20s' },
        { target: 500, duration: '20s' },
        { target: 500, duration: '30s' },
        { target: 50, duration: '20s' }
      ]
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.03'],
    http_req_duration: [`p(95)<${P95}`]
  }
};

export default function () {
  const res = http.get(`${BASE}/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38`);
  check(res, { 'status 200': (r) => r.status === 200 || r.status === 304 });
}
