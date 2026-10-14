import test from 'node:test';
import assert from 'node:assert/strict';
import { auditBapStack } from './bap-stack-audit.mjs';

const SOURCE = 'const BAP_OWNER_STACK_BYTES: usize = 16 * 1_024;\n        .stack_size(BAP_OWNER_STACK_BYTES)\n';
const OWNER = 'bitaxe_firmware::bap_adapter::run_owner';
// Synthetic BAP topology; [name, frame bytes, direct callees].
function graph(sizes = {}) {
  return [
    [OWNER, 640, ['request', 'publish', 'core::panicking::panic_fmt']],
    ['request', 1200, ['snapshot']],
    ['publish', 900, ['snapshot']],
    ['snapshot', 7000, []],
    ['core::panicking::panic_fmt', 9000, []],
  ].map(([name, bytes, callees]) => [name, sizes[name] ?? bytes, callees]);
}
function disassembly(rows) {
  const address = new Map(rows.map(([name], index) => [name, (0x42000000 + index * 0x100).toString(16)]));
  return rows.map(([name, bytes, callees]) => `${address.get(name)} <${name}>:\n ${address.get(name)}: 004136 entry a1, 0x${bytes.toString(16)}\n` +
    callees.map((callee) => ` ${address.get(name)}: 000005 call8 ${address.get(callee)} <${callee}>\n`).join('')).join('\n');
}

test('a fitting BAP path passes with reported headroom', () => {
  // Arrange / Act
  const result = auditBapStack(disassembly(graph()), SOURCE);
  // Assert
  assert.equal(result.result, 'selected_path_with_headroom');
  assert.equal(result.deepest_normal_path_bytes, 640 + 1200 + 7000 + 256);
  assert.equal(result.headroom_bytes, 16384 - 2048 - result.deepest_normal_path_bytes);
});

test('a BAP path deeper than the stack budget is blocked', () => {
  // Arrange
  const deep = graph({ request: 8000 });
  // Act
  const result = auditBapStack(disassembly(deep), SOURCE);
  // Assert
  assert.equal(result.result, 'blocked');
  assert.ok(result.headroom_bytes < 0);
});

test('the budget follows the stack size the source declares', () => {
  // Arrange
  const larger = SOURCE.replace('16 * 1_024', '20 * 1_024');
  // Act
  const result = auditBapStack(disassembly(graph({ request: 8000 })), larger);
  // Assert
  assert.equal(result.bap_stack_bytes, 20480);
  assert.equal(result.result, 'selected_path_with_headroom');
});

test('a source without the stack contract is refused', () => {
  assert.throws(() => auditBapStack(disassembly(graph()), 'const OTHER: usize = 1;'), /bap_stack_contract/u);
});
