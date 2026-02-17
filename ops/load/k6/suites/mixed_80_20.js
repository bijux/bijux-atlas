import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';
const P95 = Number(__ENV.P95_MS || 800);
const FAIL = Number(__ENV.FAIL_RATE_MAX || 0.01);

export const options = {
  scenarios: {
    mixed: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 200),
      timeUnit: '1s',
      duration: __ENV.DURATION || '90s',
      preAllocatedVUs: Number(__ENV.PRE_ALLOCATED_VUS || 64),
      maxVUs: Number(__ENV.MAX_VUS || 256)
    }
  },
  thresholds: {
    http_req_failed: [`rate<${FAIL}`],
    http_req_duration: [`p(95)<${P95}`]
  }
};

const CHEAP = [
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1',
  '/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38'
];
const HEAVY = [
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=G&limit=100',
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-100000&limit=100'
];

export default function () {
  const roll = Math.random();
  const path = roll < 0.8 ? CHEAP[Math.floor(Math.random() * CHEAP.length)] : HEAVY[Math.floor(Math.random() * HEAVY.length)];
  const res = http.get(`${BASE}${path}`);
  check(res, {
    'status ok': (r) => r.status === 200 || r.status === 304,
  });
  sleep(0.001);
}
