import test from 'node:test';
import assert from 'node:assert/strict';
import { auditWriterStack } from './writer-stack-audit.mjs';

const OWNER = 'std::thread::Builder::new()\n        .name("bwg-serial-writer".into())\n        .stack_size(8192)\n';
const WRITER = 'bitaxe_firmware::bwg_worker_usb::writer::run';
// Synthetic link topology; [name, frame bytes, direct callees].
function graph(sizes = {}) {
  return [
    [WRITER, 640, ['hello', 'close', 'core::panicking::panic_fmt']],
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

test('a fitting writer path passes with reported headroom', () => {
  // Arrange / Act
  const result = auditWriterStack(disassembly(graph()), OWNER);
  // Assert
  assert.equal(result.result, 'selected_path_with_headroom');
  assert.equal(result.deepest_normal_path_bytes, 640 + 1200 + 900 + 256);
  assert.equal(result.headroom_bytes, 6144 - result.deepest_normal_path_bytes);
});

test('panic edges are excluded rather than credited as normal depth', () => {
  // Arrange / Act
  const result = auditWriterStack(disassembly(graph()), OWNER);
  // Assert
  assert.equal(result.deepest_symbols.some((name) => /panic/u.test(name)), false);
});

test('a deep writer path over the budget is blocked', () => {
  // Arrange
  const rows = graph({ 'serde_json::to_value': 4200 });
  // Act
  const result = auditWriterStack(disassembly(rows), OWNER);
  // Assert
  assert.equal(result.result, 'blocked');
});

test('a changed writer thread stack size invalidates the audit contract', () => {
  // Arrange
  const owner = OWNER.replace('8192', '12288');
  // Act / Assert
  assert.throws(() => auditWriterStack(disassembly(graph()), owner), /writer_stack_contract/);
});

test('a call target spilled to a stack slot still counts toward depth', () => {
  // Arrange: the writer loop only loads `receive` from a literal, then calls through a reloaded register.
  const rows = [[WRITER, 640, []], ['receive', 3000, []]];
  const address = new Map(rows.map(([name], index) => [name, (0x42000000 + index * 0x100).toString(16)]));
  const text = `${address.get(WRITER)} <${WRITER}>:\n ${address.get(WRITER)}: 004136 entry a1, 0x280\n` +
    ` 42000003: 3d8f81 l32r a8, 42f00000 <_stext+0x10> (${address.get('receive')} <receive>)\n` +
    ` 42000006: 3189 s32i.n a8, a1, 12\n 42000008: 3188 l32i.n a8, a1, 12\n 4200000a: 0008e0 callx8 a8\n\n` +
    `${address.get('receive')} <receive>:\n ${address.get('receive')}: 004136 entry a1, 0xbb8\n`;
  // Act
  const result = auditWriterStack(text, OWNER);
  // Assert
  assert.equal(result.deepest_normal_path_bytes, 640 + 3000 + 256);
});
