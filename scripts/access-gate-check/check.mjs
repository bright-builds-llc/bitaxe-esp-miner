// Pure checks and the one bounded request sequence behind `just access-gate-check`.
// The result holds statuses and categories only, so it is safe to promote; the capture stays private.
import { get } from 'node:http';

const PUBLIC_ORIGIN = 'http://8.8.8.8';
const ROUTE = '/api/system/info';
const REQUEST_TIMEOUT_MS = 10_000;
const DENIED_LINE_POLLS = 15;
const DENIED_LINE_POLL_MS = 1_000;

function ipv4Category(address) {
  const octets = address.split('.').map(Number);
  if (octets.every(octet => octet === 0)) return 'zero';
  const [first, second] = octets;
  const isPrivate = first === 10 || (first === 172 && second >= 16 && second <= 31) || (first === 192 && second === 168);
  return isPrivate ? 'private' : 'public';
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
  if (!match || ipv4Category(match[1]) !== 'private') throw Error('access_gate_check_device_origin_not_private');
  return origin;
}

export function deniedPeerCategory(text) {
  const lines = text.split(/\r?\n/u).filter(line => line.includes('axeos_access_gate=denied'));
  const line = lines.at(-1);
  if (line === undefined) return 'not_logged';
  if (/\brequest_ip=None\b/u.test(line)) return 'unavailable';
  const match = /\brequest_ip=(?:Some\()?(\d{1,3}(?:\.\d{1,3}){3})\b/u.exec(line);
  return match ? ipv4Category(match[1]) : 'not_logged';
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

export async function runCheck({ readLog, requestStatus: request, sleep, expectPeer }) {
  const origin = deviceOriginFromLog(await readLog());
  const noOriginStatus = await request(origin);
  const privateOriginStatus = await request(origin, origin);

  const logLengthBefore = (await readLog()).length;
  const publicOriginStatus = await request(origin, PUBLIC_ORIGIN);
  let deniedPeer = 'not_logged';
  for (let poll = 0; poll < DENIED_LINE_POLLS && deniedPeer === 'not_logged'; poll++) {
    deniedPeer = deniedPeerCategory((await readLog()).slice(logLengthBefore));
    if (deniedPeer === 'not_logged') await sleep(DENIED_LINE_POLL_MS);
  }

  return {
    passed: noOriginStatus === 200 && privateOriginStatus === 200 && publicOriginStatus === 401 && deniedPeer === expectPeer,
    no_origin_status: noOriginStatus,
    private_origin_status: privateOriginStatus,
    public_origin_status: publicOriginStatus,
    denied_peer: deniedPeer,
    expected_peer: expectPeer,
  };
}
