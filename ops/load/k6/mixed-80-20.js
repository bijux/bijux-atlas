import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    steady_read: {
      executor: 'constant-vus',
      vus: 20,
      duration: '2m'
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<800']
  }
};

const BASE = __ENV.ATLAS_BASE || 'http://127.0.0.1:3000';

export default function () {
  const genes = http.get(`${BASE}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1`);
  check(genes, { 'genes status 200': (r) => r.status === 200 || r.status === 304 });

  const count = http.get(`${BASE}/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38`);
  check(count, { 'count status 200': (r) => r.status === 200 });

  sleep(0.2);
}
