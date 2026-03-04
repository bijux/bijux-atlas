import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    dataset_churn: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 140),
      timeUnit: '1s',
      duration: __ENV.DURATION || '2m',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 40),
      maxVUs: Number(__ENV.MAX_VUS || 180)
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.10']
  }
};

const RELEASES = ['109', '110', '111'];

export default function () {
  const release = RELEASES[Math.floor(Math.random() * RELEASES.length)];
  const res = http.get(
    `${BASE}/v1/genes/count?release=${release}&species=homo_sapiens&assembly=GRCh38`
  );
  check(res, {
    'dataset churn status acceptable': (r) => [200, 304, 429, 503].includes(r.status),
  });
  sleep(0.01);
}
