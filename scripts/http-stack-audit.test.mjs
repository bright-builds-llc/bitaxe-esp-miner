import test from 'node:test';
import assert from 'node:assert/strict';
import { auditHttpStack, DISPATCH, HANDLER, INLINED_HANDLER, QUEUED_WORK } from './http-stack-audit.mjs';

const SOURCE = 'const HTTP_SERVER_TASK_STACK_BYTES: usize = 16 * 1024;\n        stack_size: HTTP_SERVER_TASK_STACK_BYTES,\n';
// [key, label, frame bytes, callee keys]; two handler instances share one label, as in firmware.
function graph(sizes = {}) {
  return [
    ...DISPATCH.map((name) => [name, name, 64, []]),
    ['small', HANDLER, 32, ['gate']],
    ['deep', HANDLER, 32, ['snapshot', 'core::panicking::panic_fmt']],
    ['job', QUEUED_WORK[0], 48, ['persist']],
    ['persist', 'persist', 4000, []],
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

test('a queued job deeper than every handler sets the deepest path', () => {
  // Arrange: httpd_thread (64) + job (48) + persist (12000) exceeds the handler path.
  const deep = graph({ persist: 12000 });
  // Act
  const result = auditHttpStack(disassembly(deep), SOURCE);
  // Assert
  assert.equal(result.deepest_normal_path_bytes, 64 + 48 + 12000 + 256);
  assert.ok(result.deepest_symbols.includes('persist'));
});

test('a missing queued job is refused', () => {
  // Arrange
  const rows = graph().filter(([key]) => key !== 'job');
  // Act / Assert
  assert.throws(() => auditHttpStack(disassembly(rows), SOURCE), /http_queued_work_missing/u);
});

test('handlers inlined into the native trampoline are still audited', () => {
  // Arrange: the deep handler carries the inlined trampoline name.
  const rows = graph({ snapshot: 10000 }).map(([key, label, bytes, callees]) =>
    [key, key === 'deep' ? INLINED_HANDLER : label, bytes, callees]);
  // Act
  const result = auditHttpStack(disassembly(rows), SOURCE);
  // Assert
  assert.equal(result.handlers, 2);
  assert.ok(result.deepest_symbols.includes('snapshot'));
});
