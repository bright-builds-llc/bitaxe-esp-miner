import assert from 'node:assert/strict';
import test from 'node:test';
import { auditTelemetryStack } from './telemetry-stack-audit.mjs';
const sdkconfig = 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384\n';
const symbols = [
  'bitaxe_firmware::main',
  'bitaxe_firmware::http_api::cadence_owner::PreparedHttpRuntime::run',
  'bitaxe_firmware::http_api::websocket::live_telemetry_cadence_loop',
  'bitaxe_worker_control::cadence::run_iteration',
  '<bitaxe_firmware::http_api::websocket::TelemetryIteration as bitaxe_worker_control::cadence::CadenceLoopIo>::live',
  'bitaxe_firmware::runtime_snapshot::publish_projected_live_telemetry_payload_profiled',
  'bitaxe_firmware::operator_snapshot_publication::OperatorSnapshotPublisher::publish_profiled::{{closure}}',
  'bitaxe_api::runtime_projection::project_api_views',
  'bitaxe_api::wire::SystemInfoWire::from_snapshot',
  'bitaxe_api::mining::mining_state_from_runtime',
  '<alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter',
];
const frames = [64,48,48,224,208,5552,5968,1808,2224,224,96];
const hex = value => value.toString(16);
const base = index => 0x42000000 + index * 0x100;
function fixture({ inline = false, publisher = false, publisherFrame = 7392, edit = (_index, instructions) => instructions } = {}) {
  const names = [...symbols], sizes = [...frames];
  if (inline) { names.splice(6,1); sizes.splice(6,1); sizes[5] = 7344; }
  if (publisher) { names[6] = 'bitaxe_firmware::operator_snapshot_publication::OperatorSnapshotPublisher::publish_profiled'; sizes[6] = publisherFrame; sizes[5] = 112; }
  return names.map((name,index) => {
    const instructions = [`entry a1, ${sizes[index]}`];
    if (index < names.length-1) instructions.push(`l32r a3, 42009000 <literal> (${hex(base(index+1))} <${names[index+1]}>)`, 'callx8 a3');
    instructions.push('retw.n');
    return `${hex(base(index))} <${name}>:\n` + edit(index,instructions).map((line,offset)=>`${hex(base(index)+offset*3)}: 000000 ${line}\n`).join('');
  }).join('\n');
}

test('retained native frame values reproduce the 16464-byte targeted-path breach', () => {
  // Arrange / Act
  const result = auditTelemetryStack(fixture(), sdkconfig);
  // Assert
  assert.equal(result.result, 'budget_exceeded'); assert.equal(result.targeted_path_bytes, 16464);
  assert.equal(result.nodes.length, 11); assert.equal(result.complete_callgraph_bound, false);
});

test('demonstrated direct projection edge counts the corrected inlined path without a fictitious closure', () => {
  // Arrange / Act
  const result = auditTelemetryStack(fixture({inline:true}), sdkconfig);
  // Assert
  assert.equal(result.result, 'targeted_path_fits'); assert.equal(result.targeted_path_bytes, 12288);
  assert.equal(result.nodes.length, 10); assert.equal(result.hardware_safety_verified, false);
});

test('same-name unreachable large closure is not substituted for the actual callee', () => {
  // Arrange
  const extra = `43000000 <${symbols[6]}>:\n43000000: 000000 entry a1, 16384\n43000003: 000000 retw.n\n`;
  // Act / Assert
  assert.equal(auditTelemetryStack(fixture()+extra,sdkconfig).targeted_path_bytes,16464);
});

test('caller low registers survive another windowed call while high registers do not', () => {
  // Arrange
  const preserved = fixture({edit:(index,ins)=>index===5 ? [ins[0],ins[1],'callx8 a8',...ins.slice(2)] : ins});
  const clobbered = fixture({edit:(index,ins)=>index===5 ? [ins[0],ins[1].replace('a3,','a8,'),'callx8 a2','callx8 a8','retw.n'] : ins});
  // Act / Assert
  assert.equal(auditTelemetryStack(preserved,sdkconfig).targeted_path_bytes,16464);
  assert.throws(()=>auditTelemetryStack(clobbered,sdkconfig), /telemetry_path_unknown/u);
});

test('conflicting branch definitions never prove a single target at the merge', () => {
  // Arrange
  const text = fixture({edit:(index,ins)=>index===5 ? [ins[0],ins[1],`beqz a2, ${hex(base(index)+12)} <branch>`,
    'movi a3, 0','callx8 a3','retw.n'] : ins});
  // Act / Assert
  assert.throws(()=>auditTelemetryStack(text,sdkconfig), /telemetry_path_unknown/u);
});

test('unreachable literal and call do not establish a selected chain edge', () => {
  // Arrange
  const text = fixture({edit:(index,ins)=>index===5 ? [ins[0],'retw.n',...ins.slice(1)] : ins});
  // Act / Assert
  assert.throws(()=>auditTelemetryStack(text,sdkconfig), /telemetry_path_unknown/u);
});

for (const [name, edit] of [
  ['unresolved indirect target', ins=>[ins[0],'callx8 a3','retw.n']],
  ['tail jump', ins=>[ins[0],ins[1],'jx a3']],
  ['dynamic stack', ins=>[ins[0],'addi a1, a1, -16',...ins.slice(1)]],
  ['missing entry', ins=>ins.slice(1)],
  ['multiple entries', ins=>[ins[0],ins[0],...ins.slice(1)]],
]) {
  test(`unknown or unsupported ${name} fails closed`,()=> {
    // Arrange / Act / Assert
    assert.throws(()=>auditTelemetryStack(fixture({edit:(index,ins)=>index===5?edit(ins):ins}),sdkconfig), /telemetry_/u);
  });
}

test('changed or duplicate stack contract cannot silently change the budget', () => {
  // Arrange / Act / Assert
  for (const config of [sdkconfig.replace('16384','24576'),sdkconfig+sdkconfig,'']) assert.throws(()=>auditTelemetryStack(fixture(),config),/telemetry_stack_contract/u);
});

test('ordinary stores preserve target registers but compare-and-store overwrites them', () => {
  // Arrange
  const stored = fixture({edit:(index,ins)=>index===5 ? [ins[0],ins[1],'s32i a3, a1, 0',...ins.slice(2)] : ins});
  const overwritten = fixture({edit:(index,ins)=>index===5 ? [ins[0],ins[1],'s32c1i a3, a2, 0',...ins.slice(2)] : ins});
  // Act / Assert
  assert.equal(auditTelemetryStack(stored,sdkconfig).targeted_path_bytes,16464);
  assert.throws(()=>auditTelemetryStack(overwritten,sdkconfig), /telemetry_path_unknown/u);
});

test('outlined publisher detour counts the entire emitted method frame', () => {
  // Arrange / Act
  const result = auditTelemetryStack(fixture({publisher:true}), sdkconfig);
  // Assert
  assert.equal(result.result, 'targeted_path_fits');
  assert.equal(result.targeted_path_bytes, 12448);
  assert.equal(result.nodes[6].entry_bytes, 7392);
  assert.match(result.nodes[6].symbol, /::publish_profiled$/u);
  assert.equal(result.edges[5].target, hex(base(6)));
  assert.equal(result.edges[6].target, hex(base(7)));
});

test('oversized outlined publisher remains a measured budget failure', () => {
  // Arrange / Act
  const result = auditTelemetryStack(fixture({publisher:true,publisherFrame:16384}), sdkconfig);
  // Assert
  assert.equal(result.result, 'budget_exceeded');
  assert.equal(result.targeted_path_bytes, 21440);
});

test('unrecognized publisher wrapper is never skipped to reach projection', () => {
  // Arrange
  const text = fixture({publisher:true}).replaceAll('OperatorSnapshotPublisher::publish_profiled', 'OperatorSnapshotPublisher::unrecognized_wrapper');
  // Act / Assert
  assert.throws(()=>auditTelemetryStack(text,sdkconfig), /telemetry_path_unknown/u);
});

test('clobbered outlined publisher callee fails closed', () => {
  // Arrange
  const text = fixture({publisher:true,edit:(index,ins)=>index===6 ? [ins[0],ins[1].replace('a3,','a8,'),'callx8 a2','callx8 a8','retw.n'] : ins});
  // Act / Assert
  assert.throws(()=>auditTelemetryStack(text,sdkconfig), /telemetry_path_unknown/u);
});

test('outlined publisher with dynamic stack adjustment cannot claim a bound', () => {
  // Arrange
  const text = fixture({publisher:true,edit:(index,ins)=>index===6 ? [ins[0],'addi a1, a1, -16',...ins.slice(1)] : ins});
  // Act / Assert
  assert.throws(()=>auditTelemetryStack(text,sdkconfig), /telemetry_dynamic_stack_unknown/u);
});
