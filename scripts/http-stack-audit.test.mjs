import test from 'node:test';
import assert from 'node:assert/strict';
import { auditHttpStack, DISPATCH, HANDLER } from './http-stack-audit.mjs';

const SOURCE = 'const HTTP_SERVER_TASK_STACK_BYTES: usize = 16 * 1024;\n        stack_size: HTTP_SERVER_TASK_STACK_BYTES,\n';
// [key, label, frame bytes, callee keys]; two handler instances share one label, as in firmware.
function graph(sizes = {}) {
  return [
    ...DISPATCH.map((name) => [name, name, 64, []]),
    ['small', HANDLER, 32, ['gate']],
    ['deep', HANDLER, 32, ['snapshot', 'core::panicking::panic_fmt']],
    ['gate', 'gate', 96, []],
    ['snapshot', 'snapshot', 9000, []],
    ['core::panicking::panic_fmt', 'core::panicking::panic_fmt', 20000, []],
  ].map(([key, label, bytes, callees]) => [key, label, sizes[key] ?? bytes, callees]);
}
function disassembly(rows) {
  const address = new Map(rows.map(([key], index) => [key, (0x42000000 + index * 0x100).toString(16)]));
  return rows.map(([key, label, bytes, callees]) => `${address.get(key)} <${label}>:\n ${address.get(key)}: 004136 entry a1, 0x${bytes.toString(16)}\n` +
    callees.map((callee) => ` ${address.get(key)}: 000005 call8 ${address.get(callee)} <${rows.find((row) => row[0] === callee)[1]}>\n`).join('')).join('\n');
}

test('the deepest handler instance plus dispatch is charged', () => {
  // Arrange / Act
  const result = auditHttpStack(disassembly(graph()), SOURCE);
  // Assert
  assert.equal(result.handlers, 2);
  assert.equal(result.dispatch_bytes, DISPATCH.length * 64);
  assert.equal(result.deepest_normal_path_bytes, DISPATCH.length * 64 + 32 + 9000 + 256);
  assert.equal(result.result, 'selected_path_with_headroom');
});

test('a handler path past the budget is blocked', () => {
  // Arrange
  const deep = graph({ snapshot: 14000 });
  // Act
  const result = auditHttpStack(disassembly(deep), SOURCE);
  // Assert
  assert.equal(result.result, 'blocked');
  assert.ok(result.headroom_bytes < 0);
});

test('panic edges are excluded rather than credited as normal depth', () => {
  // Arrange / Act
  const result = auditHttpStack(disassembly(graph()), SOURCE);
  // Assert
  assert.equal(result.deepest_symbols.some((name) => /panic/u.test(name)), false);
});

test('a missing dispatch frame is refused', () => {
  // Arrange
  const rows = graph().filter(([key]) => key !== 'httpd_uri');
  // Act / Assert
  assert.throws(() => auditHttpStack(disassembly(rows), SOURCE), /http_dispatch_frame_missing/u);
});

test('a source without the stack contract is refused', () => {
  assert.throws(() => auditHttpStack(disassembly(graph()), 'const OTHER: usize = 1;'), /http_stack_contract/u);
});
