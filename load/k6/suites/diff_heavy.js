import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    diff_heavy: {
      executor: 'constant-arrival-rate',
      rate: 120,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 40,
      maxVUs: 160,
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<1400'],
  },
};

const base = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export default function () {
  const geneDiff = `${base}/v1/diff/genes?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&limit=50`;
  const regionDiff = `${base}/v1/diff/region?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&region=chr1:1-1000000&limit=50`;
  const url = Math.random() < 0.7 ? geneDiff : regionDiff;
  const res = http.get(url);
  check(res, { 'status ok': (r) => r.status === 200 || r.status === 304 });
  sleep(0.02);
}
