import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { deviceOriginFromLog, requestStatus, runCheck } from './check.mjs';
import { argumentsFor } from './main.mjs';

const originLine = url => `runtime_origin session=abc boot_ordinal=1 device_url=${url} redacted=true\r`;

function fakeDevice({ none = 200, privateOrigin = 200, publicOrigin = 401 }) {
  return {
    readLog: async () => originLine('http://192.168.1.50'),
    requestStatus: async (origin, maybeOriginHeader) => {
      if (maybeOriginHeader === undefined) return none;
      return maybeOriginHeader === origin ? privateOrigin : publicOrigin;
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

test('a private peer served with and without a private Origin and refused with a public one passes', async () => {
  // Arrange
  const device = fakeDevice({});
  // Act
  const result = await runCheck(device);
  // Assert
  assert.deepEqual(result, {
    passed: true,
    no_origin_status: 200,
    private_origin_status: 200,
    public_origin_status: 401,
  });
});

test('a peer refused without an Origin fails', async () => {
  // Arrange
  const device = fakeDevice({ none: 401 });
  // Act
  const result = await runCheck(device);
  // Assert
  assert.equal(result.passed, false);
});

test('a peer refused with a private Origin fails', async () => {
  // Arrange
  const device = fakeDevice({ privateOrigin: 401 });
  // Act
  const result = await runCheck(device);
  // Assert
  assert.equal(result.passed, false);
});

test('a public Origin that is served fails', async () => {
  // Arrange
  const device = fakeDevice({ publicOrigin: 200 });
  // Act
  const result = await runCheck(device);
  // Assert
  assert.equal(result.passed, false);
});

test('the output never carries the device address', async () => {
  // Arrange
  const device = fakeDevice({});
  // Act
  const result = await runCheck(device);
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

test('the arguments need exactly one capture', () => {
  // Arrange
  const good = ['--monitor-log', 'capture.log'];
  // Act
  const parsed = argumentsFor(good);
  // Assert
  assert.deepEqual(parsed, { monitorLog: 'capture.log' });
  assert.throws(() => argumentsFor([]), /access_gate_check_arguments/u);
  assert.throws(() => argumentsFor(['--other', 'x']), /access_gate_check_arguments/u);
});
