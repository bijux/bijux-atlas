import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    stampede: {
      executor: 'per-vu-iterations',
      vus: Number(__ENV.VUS || 100),
      iterations: 1,
      maxDuration: '30s'
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.10']
  }
};

export default function () {
  const res = http.get(`${BASE}/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38`);
  check(res, {
    'status acceptable': (r) => [200, 304, 503].includes(r.status),
  });
}
