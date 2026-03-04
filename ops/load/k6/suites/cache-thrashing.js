import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    cache_thrashing: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 180),
      timeUnit: '1s',
      duration: __ENV.DURATION || '2m',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 48),
      maxVUs: Number(__ENV.MAX_VUS || 220)
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.10']
  }
};

export default function () {
  const randomGene = Math.floor(Math.random() * 20000) + 1;
  const res = http.get(
    `${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=gene${randomGene}&limit=1`
  );
  check(res, {
    'cache thrashing status acceptable': (r) => [200, 304, 429, 503].includes(r.status),
  });
  sleep(0.002);
}
