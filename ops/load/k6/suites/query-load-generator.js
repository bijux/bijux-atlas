import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    query_load_generator: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 180),
      timeUnit: '1s',
      duration: __ENV.DURATION || '2m',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 48),
      maxVUs: Number(__ENV.MAX_VUS || 180)
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.08']
  }
};

const QUERY_PATHS = [
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1',
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=50',
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-100000&limit=100'
];

export default function () {
  const path = QUERY_PATHS[Math.floor(Math.random() * QUERY_PATHS.length)];
  const res = http.get(`${BASE}${path}`);
  check(res, {
    'query generator status acceptable': (r) => [200, 304, 429].includes(r.status),
  });
  sleep(0.01);
}
