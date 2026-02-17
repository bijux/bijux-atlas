import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    mixed: {
      executor: 'constant-arrival-rate',
      rate: 200,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<1200'],
  },
};

const base = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const dataset = 'release=110&species=homo_sapiens&assembly=GRCh38';

export default function () {
  const r = Math.random();
  let res;
  if (r < 0.8) {
    res = http.get(`${base}/v1/genes?${dataset}&limit=20`);
  } else {
    res = http.get(`${base}/v1/sequence/region?${dataset}&region=chr1:1-200&include_stats=1`, {
      headers: { 'x-api-key': 'load-test-key' },
    });
  }
  check(res, {
    'status acceptable': (x) => x.status === 200 || x.status === 304,
  });
  sleep(0.02);
}
