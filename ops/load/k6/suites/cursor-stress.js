import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    cursor_stress: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 160),
      timeUnit: '1s',
      duration: __ENV.DURATION || '2m',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 48),
      maxVUs: Number(__ENV.MAX_VUS || 200)
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.08']
  }
};

const FIRST_PAGE =
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=25';

export default function () {
  const first = http.get(`${BASE}${FIRST_PAGE}`);
  check(first, {
    'cursor first page status acceptable': (r) => [200, 304, 429].includes(r.status),
  });

  if (first.status === 200) {
    try {
      const body = first.json();
      if (body && body.next_cursor) {
        const second = http.get(
          `${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=25&cursor=${encodeURIComponent(body.next_cursor)}`
        );
        check(second, {
          'cursor second page status acceptable': (r) => [200, 304, 429].includes(r.status),
        });
      }
    } catch (_) {
      // tolerate non-json responses from failure paths
    }
  }
  sleep(0.005);
}
