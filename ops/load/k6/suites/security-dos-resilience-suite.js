import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    dos_resilience: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '20s', target: Number(__ENV.PEAK_VUS || 80) },
        { duration: '20s', target: Number(__ENV.PEAK_VUS || 80) },
        { duration: '20s', target: 0 }
      ]
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.3']
  }
};

const TARGET = '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=500';

export default function () {
  const res = http.get(`${BASE}${TARGET}`);
  check(res, {
    'service sheds instead of crashing': (r) => [200, 400, 401, 403, 413, 422, 429, 503].includes(r.status),
    'no 500 under pressure': (r) => r.status !== 500
  });
  sleep(0.01);
}
