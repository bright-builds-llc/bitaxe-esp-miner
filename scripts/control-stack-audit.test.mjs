import test from 'node:test';
import assert from 'node:assert/strict';
import { auditControlStack } from './control-stack-audit.mjs';

const OWNER_SOURCE = 'const OWNER_STACK_BYTES: usize = 16 * 1024;\n';
const OWNER = 'bitaxe_firmware::bwg_worker_usb::run_owner';
const FRAME = 'bitaxe_worker_control::controller::frame::<impl bitaxe_worker_control::controller::WorkerControl<V,S>>::prepare_frame';
const ROUTER = 'bitaxe_worker_control::controller::WorkerControl<V,S>::prepare_controller';
const V2 = 'bitaxe_worker_control::controller::v2::<impl bitaxe_worker_control::controller::WorkerControl<V,S>>::prepare_v2';
const GENERAL = 'bitaxe_worker_control::controller::WorkerControl<V,S>::prepare_general';
const ADMIT = 'bitaxe_worker_control::controller::v2::<impl bitaxe_worker_control::controller::WorkerControl<V,S>>::admit_v2';
// Corrected control topology with synthetic frame sizes; [name, frame bytes, direct callees].
function graph(sizes = {}, extra = []) {
  return [
    [OWNER, 688, [FRAME]],
    [FRAME, 448, [ROUTER]],
    [ROUTER, 96, [V2, GENERAL]],
    [V2, 32, [ADMIT]],
    [ADMIT, 2112, ['session_v2_admit']],
    ['session_v2_admit', 1264, []],
    [GENERAL, 3600, ['snapshot', 'core::panicking::panic_fmt']],
    ['snapshot', 3456, []],
    ['core::panicking::panic_fmt', 9000, []],
    ...extra,
  ].map(([name, bytes, callees]) => [name, sizes[name] ?? bytes, callees]);
}
function disassembly(rows) {
  const address = new Map(rows.map(([name], index) => [name, (0x42000000 + index * 0x100).toString(16)]));
  return rows.map(([name, bytes, callees]) => `${address.get(name)} <${name}>:\n ${address.get(name)}: 004136 entry a1, 0x${bytes.toString(16)}\n` +
    callees.map((callee) => ` ${address.get(name)}: 000005 call8 ${address.get(callee)} <${callee}>\n`).join('')).join('\n');
}

test('small routing frames and a fitting deepest path pass with reported headroom', () => {
  // Arrange / Act
  const result = auditControlStack(disassembly(graph()), OWNER_SOURCE);
  // Assert
  assert.equal(result.result, 'selected_path_with_headroom');
  assert.equal(result.deepest_normal_path_bytes, 688 + 448 + 96 + 3600 + 3456 + 256);
  assert.equal(result.headroom_bytes, 14336 - result.deepest_normal_path_bytes);
});

test('a route target inlined into the shared router is blocked even when depth fits', () => {
  // Arrange
  const rows = graph({ [ROUTER]: 2032 });
  // Act
  const result = auditControlStack(disassembly(rows), OWNER_SOURCE);
  // Assert
  assert.equal(result.result, 'blocked');
  assert.equal(result.routing.find((item) => item.symbol === ROUTER).fit, false);
});

test('panic edges are excluded rather than credited as normal depth', () => {
  // Arrange / Act
  const result = auditControlStack(disassembly(graph()), OWNER_SOURCE);
  // Assert
  assert.equal(result.deepest_symbols.some((name) => /panic/u.test(name)), false);
});

test('a deep normal path over the budget is blocked', () => {
  // Arrange
  const rows = graph({ snapshot: 9500 });
  // Act
  const result = auditControlStack(disassembly(rows), OWNER_SOURCE);
  // Assert
  assert.equal(result.result, 'blocked');
  assert(result.headroom_bytes < 0);
});

test('a changed control stack contract cannot pass', () => {
  // Arrange / Act / Assert
  assert.throws(() => auditControlStack(disassembly(graph()), 'const OWNER_STACK_BYTES: usize = 24 * 1024;\n'), /control_stack_contract/u);
});
