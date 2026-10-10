import test from 'node:test';
import assert from 'node:assert/strict';
import { auditLinkStack } from './link-stack-audit.mjs';

const PREPARED = 'prepared_thread::spawn("bwg-serial-link", 8192, Box::new(|| super::link::run()))\n';
const LINK = 'bitaxe_firmware::bwg_worker_usb::link::run';
// Synthetic link topology; [name, frame bytes, direct callees].
function graph(sizes = {}) {
  return [
    [LINK, 640, ['hello', 'close', 'core::panicking::panic_fmt']],
    ['hello', 1200, ['serde_json::to_value']],
    ['serde_json::to_value', 900, []],
    ['close', 96, []],
    ['core::panicking::panic_fmt', 9000, []],
  ].map(([name, bytes, callees]) => [name, sizes[name] ?? bytes, callees]);
}
function disassembly(rows) {
  const address = new Map(rows.map(([name], index) => [name, (0x42000000 + index * 0x100).toString(16)]));
  return rows.map(([name, bytes, callees]) => `${address.get(name)} <${name}>:\n ${address.get(name)}: 004136 entry a1, 0x${bytes.toString(16)}\n` +
    callees.map((callee) => ` ${address.get(name)}: 000005 call8 ${address.get(callee)} <${callee}>\n`).join('')).join('\n');
}

test('a fitting link path passes with reported headroom', () => {
  // Arrange / Act
  const result = auditLinkStack(disassembly(graph()), PREPARED);
  // Assert
  assert.equal(result.result, 'selected_path_with_headroom');
  assert.equal(result.deepest_normal_path_bytes, 640 + 1200 + 900 + 256);
  assert.equal(result.headroom_bytes, 6144 - result.deepest_normal_path_bytes);
});

test('panic edges are excluded rather than credited as normal depth', () => {
  // Arrange / Act
  const result = auditLinkStack(disassembly(graph()), PREPARED);
  // Assert
  assert.equal(result.deepest_symbols.some((name) => /panic/u.test(name)), false);
});

test('a deep link path over the budget is blocked', () => {
  // Arrange
  const rows = graph({ 'serde_json::to_value': 4200 });
  // Act
  const result = auditLinkStack(disassembly(rows), PREPARED);
  // Assert
  assert.equal(result.result, 'blocked');
});

test('a changed RX thread stack size invalidates the audit contract', () => {
  // Arrange
  const prepared = PREPARED.replace('8192', '12288');
  // Act / Assert
  assert.throws(() => auditLinkStack(disassembly(graph()), prepared), /link_stack_contract/);
});
