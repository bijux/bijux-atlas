import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const P95 = Number(__ENV.P95_MS || 700);

export const options = {
  scenarios: {
    steady: {
      executor: 'constant-vus',
      vus: Number(__ENV.VUS || 40),
      duration: __ENV.DURATION || '2m'
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: [`p(95)<${P95}`]
  }
};

export default function () {
  const res = http.get(`${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1`);
  check(res, { 'status 200': (r) => r.status === 200 || r.status === 304 });
  sleep(0.05);
}
