import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { deniedPeerCategory, deviceOriginFromLog, requestStatus, runCheck } from './check.mjs';
import { argumentsFor } from './main.mjs';

const originLine = url => `I (9) bitaxe: runtime_origin session=abc boot_ordinal=1 device_url=${url} redacted=true\r`;
const deniedLine = ip => `W (9) bitaxe: axeos_access_gate=denied kind=http path=/api/system/info ap_mode_enabled=false request_ip=${ip} origin=Parsed(8.8.8.8)`;

function fakeDevice({ peerLine, statuses = { none: 200, private: 200, public: 401 } }) {
  let log = originLine('http://192.168.1.50');
  const calls = [];
  return {
    calls,
    readLog: async () => log,
    sleep: async () => {},
    requestStatus: async (origin, maybeOriginHeader) => {
      calls.push({ origin, maybeOriginHeader });
      if (maybeOriginHeader === undefined) return statuses.none;
      if (maybeOriginHeader === origin) return statuses.private;
      if (peerLine) log += `\n${peerLine}`;
      return statuses.public;
    },
  };
}

test('the device origin comes from the one runtime origin in the capture', () => {
  // Arrange
  const text = [originLine('http://192.168.1.50'), 'unrelated', originLine('http://192.168.1.50')].join('\n');
  // Act
  const origin = deviceOriginFromLog(text);
  // Assert
  assert.equal(origin, 'http://192.168.1.50');
});

test('a capture with no runtime origin is refused', () => {
  // Arrange
  const text = 'usb_startup schema=v1\n';
  // Act / Assert
  assert.throws(() => deviceOriginFromLog(text), /access_gate_check_device_origin_not_unique/u);
});

test('a capture with two different runtime origins is refused', () => {
  // Arrange
  const text = [originLine('http://192.168.1.50'), originLine('http://192.168.1.51')].join('\n');
  // Act / Assert
  assert.throws(() => deviceOriginFromLog(text), /access_gate_check_device_origin_not_unique/u);
});

test('a device origin outside the private ranges is refused', () => {
  // Arrange
  const addresses = ['http://8.8.8.8', 'http://172.32.0.1', 'http://device.test', 'https://192.168.1.50', 'http://192.168.1.50/path'];
  for (const address of addresses) {
    // Act / Assert
    assert.throws(() => deviceOriginFromLog(originLine(address)), /access_gate_check_device_origin_not_private/u, address);
  }
});

test('the denied line is classified without keeping its address', () => {
  // Arrange
  const cases = [
    ['request_ip=0.0.0.0', 'zero'],
    ['request_ip=Some(0.0.0.0)', 'zero'],
    ['request_ip=None', 'unavailable'],
    ['request_ip=Some(192.168.1.25)', 'private'],
    ['request_ip=Some(10.0.0.2)', 'private'],
    ['request_ip=Some(172.16.0.2)', 'private'],
    ['request_ip=Some(8.8.8.8)', 'public'],
  ];
  for (const [field, category] of cases) {
    // Act
    const actual = deniedPeerCategory(`axeos_access_gate=denied kind=http ${field} origin=Invalid`);
    // Assert
    assert.equal(actual, category, field);
  }
});

test('no denied line reads as not logged', () => {
  // Arrange
  const text = 'unrelated\n';
  // Act
  const category = deniedPeerCategory(text);
  // Assert
  assert.equal(category, 'not_logged');
});

test('a real private peer on the fixed firmware passes', async () => {
  // Arrange
  const device = fakeDevice({ peerLine: deniedLine('Some(192.168.1.25)') });
  // Act
  const result = await runCheck({ ...device, expectPeer: 'private' });
  // Assert
  assert.deepEqual(result, {
    passed: true,
    no_origin_status: 200,
    private_origin_status: 200,
    public_origin_status: 401,
    denied_peer: 'private',
    expected_peer: 'private',
  });
});

test('a zero peer fails when a real private peer is expected', async () => {
  // Arrange
  const device = fakeDevice({ peerLine: deniedLine('0.0.0.0') });
  // Act
  const result = await runCheck({ ...device, expectPeer: 'private' });
  // Assert
  assert.equal(result.passed, false);
  assert.equal(result.denied_peer, 'zero');
});

test('a zero peer passes when the old firmware is being confirmed', async () => {
  // Arrange
  const device = fakeDevice({ peerLine: deniedLine('0.0.0.0') });
  // Act
  const result = await runCheck({ ...device, expectPeer: 'zero' });
  // Assert
  assert.equal(result.passed, true);
});

test('a private peer refused without an Origin fails', async () => {
  // Arrange
  const device = fakeDevice({
    peerLine: deniedLine('Some(192.168.1.25)'),
    statuses: { none: 401, private: 200, public: 401 },
  });
  // Act
  const result = await runCheck({ ...device, expectPeer: 'private' });
  // Assert
  assert.equal(result.passed, false);
});

test('a public Origin that is served fails', async () => {
  // Arrange
  const device = fakeDevice({
    peerLine: deniedLine('Some(192.168.1.25)'),
    statuses: { none: 200, private: 200, public: 200 },
  });
  // Act
  const result = await runCheck({ ...device, expectPeer: 'private' });
  // Assert
  assert.equal(result.passed, false);
});

test('a denied line from before the public request is not counted', async () => {
  // Arrange
  const device = fakeDevice({ peerLine: undefined });
  const readLog = device.readLog;
  device.readLog = async () => `${deniedLine('Some(192.168.1.25)')}\n${await readLog()}`;
  // Act
  const result = await runCheck({ ...device, expectPeer: 'private' });
  // Assert
  assert.equal(result.passed, false);
  assert.equal(result.denied_peer, 'not_logged');
});

test('the output never carries the device or peer address', async () => {
  // Arrange
  const device = fakeDevice({ peerLine: deniedLine('Some(192.168.1.25)') });
  // Act
  const result = await runCheck({ ...device, expectPeer: 'private' });
  // Assert
  assert.doesNotMatch(JSON.stringify(result), /192\.168/u);
});

test('a request sends the Origin it is given and returns the status', async () => {
  // Arrange
  const seen = [];
  const server = createServer((request, response) => {
    seen.push(request.headers.origin);
    response.statusCode = request.headers.origin === 'http://8.8.8.8' ? 401 : 200;
    response.end('body');
  });
  await new Promise(done => server.listen(0, '127.0.0.1', done));
  const origin = `http://127.0.0.1:${server.address().port}`;
  // Act
  const statuses = [await requestStatus(origin), await requestStatus(origin, 'http://8.8.8.8')];
  server.close();
  // Assert
  assert.deepEqual(statuses, [200, 401]);
  assert.deepEqual(seen, [undefined, 'http://8.8.8.8']);
});

test('the arguments need a capture and a known expected peer', () => {
  // Arrange
  const good = ['--monitor-log', 'capture.log', '--expect-peer', 'private'];
  // Act
  const parsed = argumentsFor(good);
  // Assert
  assert.deepEqual(parsed, { monitorLog: 'capture.log', expectPeer: 'private' });
  assert.throws(() => argumentsFor(['--monitor-log', 'capture.log']), /access_gate_check_arguments/u);
  assert.throws(() => argumentsFor(['--monitor-log', 'x', '--expect-peer', 'public']), /access_gate_check_arguments/u);
});
