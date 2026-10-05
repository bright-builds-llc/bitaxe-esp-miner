import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { auditStartupFrames, frameSizes } from './startup-frame-audit.mjs';

const budget = JSON.parse(readFileSync(new URL('./startup-frame-budget.json', import.meta.url), 'utf8'));
const fn = (address, name, frame) => `
${address} <${name}>:
${address}:\t1a2b36        \tentry\ta1, ${frame}
${address.replace(/.$/u, '3')}:\tf01d          \tretw.n
`;
const image = (runStartup, withStart = true) => [
  fn('42001000', 'bitaxe_firmware::startup::run_startup', runStartup),
  withStart ? fn('42002000', 'bitaxe_firmware::production_mining_session::start', '0x1120') : '',
].join('');

test('frame sizes read decimal and hexadecimal entry operands', () => {
  // Arrange / Act
  const frames = frameSizes(fn('42001000', 'a', '0x6f0') + fn('42002000', 'b', '48'));
  // Assert
  assert.deepEqual([frames.get('a'), frames.get('b')], [0x6f0, 48]);
});

test('the previous and corrected startup frames pass', () => {
  // Arrange / Act / Assert: 7ca3e29c measured 0x6f0; the corrected queue image 0x430.
  assert.equal(auditStartupFrames(image('0x6f0'), budget).result, 'startup_frames_within_budget');
  assert.equal(auditStartupFrames(image('0x430'), budget).result, 'startup_frames_within_budget');
});

test('the boot-looping startup frame is blocked', () => {
  // Arrange / Act: a2052ab0 measured 0x1320 after inlining the session start.
  const result = auditStartupFrames(image('0x1320', false), budget);
  // Assert
  assert.deepEqual(result.violations.map(row => row.kind).sort(), ['frame_over_budget', 'inlined']);
});

test('a missing budgeted function is blocked rather than skipped', () => {
  // Arrange / Act
  const result = auditStartupFrames(fn('42002000', 'bitaxe_firmware::production_mining_session::start', '0x1120'), budget);
  // Assert
  assert.deepEqual(result.violations, [{ symbol: 'bitaxe_firmware::startup::run_startup', kind: 'frame_missing' }]);
});
