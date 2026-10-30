// Pure checks and the one bounded request sequence behind `just access-gate-check`.
// On firmware that denies an unreadable peer, a 200 with no Origin can only mean the gate read a private
// IPv4-mapped peer. The result holds statuses only, so it is safe to promote; the capture stays private.
import { get } from 'node:http';

const PUBLIC_ORIGIN = 'http://8.8.8.8';
const ROUTE = '/api/system/info';
const REQUEST_TIMEOUT_MS = 10_000;

function isPrivateIpv4(address) {
  const [first, second] = address.split('.').map(Number);
  return first === 10 || (first === 172 && second >= 16 && second <= 31) || (first === 192 && second === 168);
}

export function deviceOriginFromLog(text) {
  const origins = new Set();
  for (const line of text.split(/\r?\n/u)) {
    if (!line.includes('runtime_origin ')) continue;
    const match = /\bdevice_url=(\S+)\s+redacted=true\b/u.exec(line);
    if (match) origins.add(match[1]);
  }
  if (origins.size !== 1) throw Error('access_gate_check_device_origin_not_unique');
  const [origin] = origins;
  const match = /^http:\/\/(\d{1,3}(?:\.\d{1,3}){3})(?::\d+)?$/u.exec(origin);
  if (!match || !isPrivateIpv4(match[1])) throw Error('access_gate_check_device_origin_not_private');
  return origin;
}

export function requestStatus(origin, maybeOriginHeader) {
  return new Promise((resolve, reject) => {
    const headers = maybeOriginHeader === undefined ? {} : { origin: maybeOriginHeader };
    const request = get(new URL(ROUTE, origin), { headers, agent: false, timeout: REQUEST_TIMEOUT_MS }, response => {
      response.resume();
      response.on('end', () => resolve(response.statusCode));
      response.on('error', reject);
    });
    request.on('timeout', () => request.destroy(Error('access_gate_check_request_timeout')));
    request.on('error', reject);
  });
}

export async function runCheck({ readLog, requestStatus: request }) {
  const origin = deviceOriginFromLog(await readLog());
  const noOriginStatus = await request(origin);
  const privateOriginStatus = await request(origin, origin);
  const publicOriginStatus = await request(origin, PUBLIC_ORIGIN);

  return {
    passed: noOriginStatus === 200 && privateOriginStatus === 200 && publicOriginStatus === 401,
    no_origin_status: noOriginStatus,
    private_origin_status: privateOriginStatus,
    public_origin_status: publicOriginStatus,
  };
}
