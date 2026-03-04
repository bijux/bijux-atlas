import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    penetration_simulation: {
      executor: 'constant-vus',
      vus: Number(__ENV.VUS || 20),
      duration: __ENV.DURATION || '90s'
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.15'],
    http_req_duration: ['p(95)<1200']
  }
};

const PATHS = [
  '/v1/version',
  '/v1/datasets',
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1',
  '/v1/query/validate'
];

export default function () {
  const p = PATHS[Math.floor(Math.random() * PATHS.length)];
  const res = http.get(`${BASE}${p}`, {
    headers: {
      'User-Agent': 'atlas-security-probe/1.0',
      'X-Forwarded-For': `198.51.100.${__VU % 255}`
    }
  });

  check(res, {
    'endpoint does not crash': (r) => r.status !== 500,
    'status is controlled': (r) => [200, 304, 400, 401, 403, 404, 413, 422, 429, 503].includes(r.status)
  });

  sleep(0.05);
}
