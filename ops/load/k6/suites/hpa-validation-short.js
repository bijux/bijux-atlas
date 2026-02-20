import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:18080';

export const options = {
  scenarios: {
    hpa_short: {
      executor: 'ramping-vus',
      startVUs: 5,
      stages: [
        { duration: '20s', target: 25 },
        { duration: '30s', target: 60 },
        { duration: '20s', target: 15 }
      ]
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.15'],
    http_req_duration: ['p(95)<2200']
  }
};

export default function () {
  const heavy = http.get(`${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=100`);
  check(heavy, { 'heavy request handled or shed': (r) => r.status === 200 || r.status === 429 || r.status === 503 });

  const cheap = http.get(`${BASE}/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38`);
  check(cheap, { 'cheap request survives': (r) => r.status === 200 || r.status === 304 });
  sleep(0.02);
}
