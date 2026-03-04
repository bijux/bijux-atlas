import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    shard_hot_spot: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 220),
      timeUnit: '1s',
      duration: __ENV.DURATION || '2m',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 64),
      maxVUs: Number(__ENV.MAX_VUS || 256)
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.08']
  }
};

export default function () {
  const res = http.get(
    `${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-200000&limit=100`
  );
  check(res, {
    'shard hot-spot status acceptable': (r) => [200, 304, 429, 503].includes(r.status),
  });
  sleep(0.005);
}
