import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    soak: {
      executor: 'constant-vus',
      vus: Number(__ENV.VUS || 25),
      duration: __ENV.DURATION || '30m'
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.02'],
    http_req_duration: ['p(95)<900']
  }
};

export default function () {
  const res = http.get(`${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=50`);
  check(res, { 'status acceptable': (r) => r.status === 200 || r.status === 304 || r.status === 429 });
  sleep(0.1);
}
