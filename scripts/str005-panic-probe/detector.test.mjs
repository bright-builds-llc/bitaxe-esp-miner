import assert from 'node:assert/strict';
import test from 'node:test';
import { parseDetector, cleanupPorts } from './detector.mjs';
const physical = 'a'.repeat(64);
const report = port => `port: ${port}\nphysical_identity_sha256: ${physical}\nusb_profile: serial_jtag_runtime\n`;
test('fresh renamed node keeps the same physical authority', () => {
  // Arrange / Act / Assert
  assert.equal(parseDetector(report('/dev/cu.usbmodem-new'), physical, 1).port, '/dev/cu.usbmodem-new');
  assert.equal(parseDetector(report('/dev/cu.usbmodem-old'), physical, 60000).physical, physical);
});
test('changed identity, unknown profile, ambiguous node and stale detector fail closed', () => {
  // Arrange / Act / Assert
  for (const document of [report('/dev/cu.a').replace(physical, 'b'.repeat(64)), report('/dev/cu.a').replace('serial_jtag_runtime', 'rom_downloader'),
    report('/dev/cu.a') + 'port: /dev/cu.b\n', report('/dev/../secret')]) assert.throws(() => parseDetector(document, physical, 1));
  for (const age of [-1, 60001, NaN]) assert.throws(() => parseDetector(report('/dev/cu.a'), physical, age));
});

test('cleanup checks fresh, server and install nodes once each', () => {
  // Arrange / Act / Assert
  assert.deepEqual(cleanupPorts('/dev/new', '/dev/old', '/dev/install'), ['/dev/new', '/dev/old', '/dev/install']);
  assert.deepEqual(cleanupPorts('/dev/new', '/dev/new', '/dev/new'), ['/dev/new']);
  assert.deepEqual(cleanupPorts('/dev/new', '/dev/old'), ['/dev/new', '/dev/old']);
  assert.throws(() => cleanupPorts('/dev/new', '/dev/old', '/tmp/unrelated'));
});
