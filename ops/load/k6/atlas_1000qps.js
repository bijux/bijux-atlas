import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    steady_1000qps: {
      executor: 'constant-arrival-rate',
      rate: 1000,
      timeUnit: '1s',
      duration: __ENV.DURATION || '60s',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 200),
      maxVUs: Number(__ENV.MAX_VUS || 400),
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<800'],
  },
};

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const DATASET = __ENV.DATASET || 'release=110&species=homo_sapiens&assembly=GRCh38';

export default function () {
  const url = `${BASE}/v1/genes/count?${DATASET}`;
  const res = http.get(url, { headers: { Accept: 'application/json' } });
  check(res, {
    'status 200': (r) => r.status === 200,
  });
  sleep(0.001);
}
