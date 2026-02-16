import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    throttle_probe: {
      executor: 'constant-vus',
      vus: Number(__ENV.VUS || 40),
      duration: __ENV.DURATION || '120s'
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.10'],
    http_req_duration: [`p(95)<${Number(__ENV.P95_MS || 1500)}`]
  }
};

export default function () {
  const cheap = http.get(`${BASE}/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38`);
  check(cheap, { 'cheap path stays available': (r) => r.status === 200 || r.status === 304 });

  const heavy = http.get(`${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=100`);
  check(heavy, { 'heavy path can degrade gracefully': (r) => r.status === 200 || r.status === 503 });
  sleep(0.01);
}
