import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    ingest_load_generator: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 90),
      timeUnit: '1s',
      duration: __ENV.DURATION || '2m',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 24),
      maxVUs: Number(__ENV.MAX_VUS || 120)
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.15']
  }
};

const INGEST_ADJACENT_PATHS = [
  '/v1/diff/genes?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&limit=50',
  '/v1/diff/region?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&region=chr1:1-100000&limit=50',
  '/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38'
];

export default function () {
  const path = INGEST_ADJACENT_PATHS[Math.floor(Math.random() * INGEST_ADJACENT_PATHS.length)];
  const res = http.get(`${BASE}${path}`);
  check(res, {
    'ingest generator status acceptable': (r) => [200, 304, 429, 503].includes(r.status),
  });
  sleep(0.02);
}
