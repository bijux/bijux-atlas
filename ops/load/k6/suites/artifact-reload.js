import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8080';

export const options = {
  scenarios: {
    artifact_reload: {
      executor: 'constant-vus',
      vus: Number(__ENV.VUS || 30),
      duration: __ENV.DURATION || '2m'
    }
  },
  thresholds: {
    http_req_failed: ['rate<0.08']
  }
};

export default function () {
  const health = http.get(`${BASE}/ready`);
  check(health, {
    'ready endpoint status acceptable': (r) => [200, 503].includes(r.status),
  });

  const version = http.get(`${BASE}/v1/version`);
  check(version, {
    'version endpoint status acceptable': (r) => [200, 304].includes(r.status),
  });
  sleep(0.03);
}
