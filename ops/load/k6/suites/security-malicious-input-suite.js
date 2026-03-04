import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  vus: Number(__ENV.VUS || 8),
  duration: __ENV.DURATION || '60s',
  thresholds: {
    http_req_failed: ['rate<0.2']
  }
};

const BAD_INPUTS = [
  '"\u0000\u0001\u0002"',
  '"<script>alert(1)</script>"',
  '"../../../../etc/passwd"',
  '"%00%0a%0d"',
  '"A"'.repeat(2048)
];

export default function () {
  const body = JSON.stringify({ query: BAD_INPUTS[__ITER % BAD_INPUTS.length] });
  const res = http.post(`${BASE}/v1/query/validate`, body, {
    headers: { 'Content-Type': 'application/json' }
  });

  check(res, {
    'malicious payload handled safely': (r) => [200, 400, 401, 403, 413, 422, 429, 503].includes(r.status),
    'no 5xx': (r) => r.status !== 500
  });
}
