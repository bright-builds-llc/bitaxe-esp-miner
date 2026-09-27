import { check } from '../str005-v2-serial/values.mjs';

/** Physical identity is authority; a fresh node is only this operation's locator. */
export function parseDetector(text, expectedPhysical, ageMs) {
  check(typeof text === 'string' && Buffer.byteLength(text) <= 1048576 && Number.isFinite(ageMs) && ageMs >= 0 && ageMs <= 60000,
    'panic_detector_stale');
  check(typeof expectedPhysical === 'string' && /^[0-9a-f]{64}$/u.test(expectedPhysical), 'panic_detector_physical');
  const rows = [...text.matchAll(/^([a-z][a-z0-9_]*): (.+)$/gmu)];
  const unique = key => {
    const matches = rows.filter(row => row[1] === key);
    check(matches.length === 1, 'panic_detector_ambiguous'); return matches[0][2];
  };
  const port = unique('port'), physical = unique('physical_identity_sha256'), profile = unique('usb_profile');
  check(physical === expectedPhysical && profile === 'serial_jtag_runtime', 'panic_detector_identity');
  check(/^\/dev\/[A-Za-z0-9._/-]+$/u.test(port) && !port.split('/').includes('..') && !port.includes('//'), 'panic_detector_port');
  return { port, physical, profile };
}

/** Check every observed locator while retaining the fresh physical-device admission. */
export function cleanupPorts(currentPort, serverPort, maybeInstallPort) {
  const ports = [currentPort, serverPort, ...(maybeInstallPort === undefined ? [] : [maybeInstallPort])];
  for (const port of ports) check(typeof port === 'string' && /^\/dev\/[A-Za-z0-9._/-]+$/u.test(port) &&
    !port.split('/').includes('..') && !port.includes('//'), 'panic_cleanup_port');
  return [...new Set(ports)];
}
