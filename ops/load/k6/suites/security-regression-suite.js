import http from 'k6/http';
import { check } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    security_regression: {
      executor: 'per-vu-iterations',
      vus: Number(__ENV.VUS || 10),
      iterations: Number(__ENV.ITERATIONS || 30),
      maxDuration: __ENV.MAX_DURATION || '2m'
    }
  }
};

const CASES = [
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1',
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&cursor=bad.cursor',
  '/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&range=chr1:1-3',
  '/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&range=chr1:1-20'
];

export default function () {
  for (const path of CASES) {
    const res = http.get(`${BASE}${path}`);
    check(res, {
      'no internal server error': (r) => r.status !== 500,
    });
  }
}
