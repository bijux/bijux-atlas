import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    http_request_generator: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 120),
      timeUnit: '1s',
      duration: __ENV.DURATION || '90s',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 32),
      maxVUs: Number(__ENV.MAX_VUS || 128)
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.10']
  }
};

const PATHS = [
  '/v1/version',
  '/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38'
];

export default function () {
  const path = PATHS[Math.floor(Math.random() * PATHS.length)];
  const res = http.get(`${BASE}${path}`);
  check(res, {
    'http generator status acceptable': (r) => [200, 304].includes(r.status),
  });
  sleep(0.01);
}
