import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    outage_spike: {
      executor: 'ramping-vus',
      startVUs: 5,
      stages: [
        { duration: '20s', target: 20 },
        { duration: '20s', target: 60 },
        { duration: '20s', target: 20 }
      ]
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.20']
  }
};

export default function () {
  const cached = http.get(`${BASE}/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38`);
  check(cached, { 'cached dataset serves': (r) => r.status === 200 || r.status === 304 });

  const uncached = http.get(`${BASE}/v1/genes/count?release=999&species=homo_sapiens&assembly=GRCh38`);
  check(uncached, { 'uncached fails fast': (r) => r.status === 400 || r.status === 503 });
}
