import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, writeFile, readFile, rm, mkdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { admitNoisePackage, parseNoiseCheckpoints, extractNoiseCheckpoints, judgeNoiseProbe, requireNoNoiseWriters, runNoiseProbe, finalizeNoiseResult } from './noise.mjs';
import { auditNoiseStack, noiseAuditIdentity } from './noise-stack-audit.mjs';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const source = 'a'.repeat(64);
const config = ['CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384', 'CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0',
  'CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304', 'CONFIG_SPIRAM_MODE_OCT=y', 'CONFIG_ESPTOOLPY_FLASHSIZE_16MB=y',
  'CONFIG_FREERTOS_CHECK_STACKOVERFLOW_CANARY=y', 'CONFIG_FREERTOS_TASK_FUNCTION_WRAPPER=y', '# CONFIG_LIBC_NEWLIB_NANO_FORMAT is not set'].join('\n');
// Corrected helper topology; synthetic 64-byte frames test admission only.
const COMPLETE = 'bitaxe_stratum::v2::noise::completion::<impl bitaxe_stratum::v2::noise::NoiseInitiator>::complete_diagnostic_into';
const graph = [
  ['bitaxe_virtual_firmware::main', ['bitaxe_virtual_firmware::guest::run']],
  ['bitaxe_virtual_firmware::guest::run', ['bitaxe_virtual_firmware::noise_probe::run_and_emit']],
  ['bitaxe_virtual_firmware::noise_probe::run_and_emit', ['bitaxe_simulation::noise_probe::run', 'bitaxe_virtual_firmware::noise_probe::emit_outcome']],
  ['bitaxe_virtual_firmware::noise_probe::emit_outcome', []],
  ['bitaxe_simulation::noise_probe::run', ['bitaxe_simulation::noise_probe::handshake_and_frame']],
  ['bitaxe_simulation::noise_probe::handshake_and_frame', ['bitaxe_simulation::noise_probe::prepare_initiator',
    'bitaxe_simulation::noise_probe::respond', COMPLETE, 'bitaxe_simulation::noise_probe::frame_round_trip']],
  ['bitaxe_simulation::noise_probe::prepare_initiator', ['noise_sv2::initiator::Initiator::new_with_rng', 'bitaxe_stratum::v2::noise::NoiseInitiator::act_one']],
  ['noise_sv2::initiator::Initiator::new_with_rng', []],
  ['bitaxe_stratum::v2::noise::NoiseInitiator::act_one', []],
  ['bitaxe_simulation::noise_probe::respond', ['bitaxe_simulation::v2::exchange::construct_responder', 'bitaxe_simulation::v2::exchange::step_responder']],
  ['bitaxe_simulation::v2::exchange::construct_responder', ['noise_sv2::responder::Responder::from_authority_kp_with_rng']],
  ['noise_sv2::responder::Responder::from_authority_kp_with_rng', []],
  ['bitaxe_simulation::v2::exchange::step_responder', ['noise_sv2::responder::Responder::step_1_with_now_rng']],
  ['noise_sv2::responder::Responder::step_1_with_now_rng', []],
  [COMPLETE, ['noise_sv2::initiator::Initiator::step_2_with_now']],
  ['noise_sv2::initiator::Initiator::step_2_with_now', ['rustsecp256k1_v0_9_2_schnorrsig_verify']],
  ['rustsecp256k1_v0_9_2_schnorrsig_verify', []],
  ['bitaxe_simulation::noise_probe::frame_round_trip', []],
  ['bitaxe_simulation::v2::exchange::Exchange::new', []],
  ['bitaxe_simulation::v2::exchange::handshake', ['bitaxe_simulation::v2::exchange::prepare_initiator',
    'bitaxe_simulation::v2::exchange::construct_responder', 'bitaxe_simulation::v2::exchange::step_responder', COMPLETE]],
  ['bitaxe_simulation::v2::exchange::prepare_initiator', ['noise_sv2::initiator::Initiator::new_with_rng']],
];
const addressOf = new Map(graph.map(([name], i) => [name, (0x40000000 + i * 256).toString(16)]));
const disassembly = graph.map(([name, callees]) => `${addressOf.get(name)} <${name}>:\n ${addressOf.get(name)}: 004136 entry a1, 64\n` +
  callees.map(callee => ` ${addressOf.get(name)}: 000005 call8 ${addressOf.get(callee)} <${callee}>\n`).join('')).join('\n');

async function fixture(root) {
  // Synthetic format fixtures test byte admission only; no target execution or allocator measurement is claimed.
  const elf = Buffer.alloc(256); elf.writeUInt32BE(0x7f454c46); elf[4] = 1; elf[5] = 1; elf.writeUInt16LE(94, 18);
  elf.write(`BITAXE_EXECUTION_PROFILE=virtual-ultra205\0${source}`, 52);
  const image = Buffer.alloc(16777216, 0xff);
  const manifest = { schema: 'bitaxe-virtual-package-v1', execution_profile: 'virtual-ultra205', hardware_eligible: false,
    sdk: 'v5.5.4', virtual_elf: 'guest.elf', flash_image: 'flash.bin', virtual_elf_sha256: digest(elf),
    image_sha256: digest(image), virtual_sdkconfig_sha256: digest(config), compiled_source_sha256: source,
    board_profile: { cores: 2, flash_bytes: 16777216, psram_bytes: 8388608, psram_mode: 'octal' } };
  const audit = auditNoiseStack(disassembly, config);
  audit.bindings = { elf_sha256: manifest.virtual_elf_sha256, sdkconfig_sha256: manifest.virtual_sdkconfig_sha256,
    compiled_source_sha256: source, auditor_sha256: await noiseAuditIdentity(), objdump_version: 'esp-14.2.0_20260121',
    objdump_sha256: digest('test-objdump-bytes'), disassembly_sha256: digest(disassembly) };
  for (const [name, bytes] of Object.entries({ 'objdump': 'test-objdump-bytes', 'guest.elf': elf, 'flash.bin': image, 'virtual-ultra205.sdkconfig': config,
    'manifest.json': JSON.stringify(manifest), 'audit.json': JSON.stringify(audit), 'audit.json.disassembly.private': disassembly })) {
    await writeFile(join(root, name), bytes, { flag: 'wx', mode: 0o600 });
  }
  return { manifest, audit, manifestPath: join(root, 'manifest.json'), auditPath: join(root, 'audit.json') };
}

for (const [name, mutate, pattern] of [
  ['physical package substitution', async (root, data) => { data.manifest.execution_profile = 'production'; await writeFile(data.manifestPath, JSON.stringify(data.manifest)); }, /noise_package_profile/],
  ['virtual ELF byte mutation', async root => { const path = join(root, 'guest.elf'), bytes = await readFile(path); bytes[200] ^= 1; await writeFile(path, bytes); }, /noise_package_digest/],
  ['resolved SDK routing mutation', async root => { await writeFile(join(root, 'virtual-ultra205.sdkconfig'), config.replace('CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0', 'CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=4096')); }, /noise_package_digest/],
  ['stale audit native ELF binding', async (root, data) => { data.audit.bindings.elf_sha256 = '0'.repeat(64); await writeFile(data.auditPath, JSON.stringify(data.audit)); }, /noise_audit_binding/],
  ['current objdump byte substitution', async root => { await writeFile(join(root, 'objdump'), 'changed-tool'); }, /noise_objdump_identity/],
  ['mutated disassembly proof', async (root, data) => { await writeFile(`${data.auditPath}.disassembly.private`, disassembly.replace('entry a1, 64', 'entry a1, 16000')); }, /noise_audit_native_proof/],
]) {
  test(`read-only admission rejects ${name}`, async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), 'noise-admission-'));
    try {
      const data = await fixture(root); await mutate(root, data);
      // Act / Assert
      await assert.rejects(admitNoisePackage(data.manifestPath, data.auditPath, source, join(root, 'objdump')), pattern);
    } finally { await rm(root, { recursive: true }); }
  });
}

test('read-only admission binds actual package/config/ELF bytes and recomputed native proof', async () => {
  // Arrange
  const root = await mkdtemp(join(tmpdir(), 'noise-admission-'));
  try {
    const data = await fixture(root);
    // Act
    const result = await admitNoisePackage(data.manifestPath, data.auditPath, source, join(root, 'objdump'));
    // Assert
    assert.equal(digest(result.elf), result.manifest.virtual_elf_sha256);
    assert.equal(digest(result.config), result.manifest.virtual_sdkconfig_sha256);
    await assert.rejects(admitNoisePackage(data.manifestPath, data.auditPath, 'c'.repeat(64), join(root, 'objdump')), /noise_package_digest|noise_source_identity/);
  } finally { await rm(root, { recursive: true }); }
});

test('disabled effect gate rejects before filesystem admission or any process', async () => {
  // Arrange
  const absent = '/nonexistent/noise-proof';
  // Act / Assert
  await assert.rejects(runNoiseProbe(absent, absent, absent, { disableEffects: true }), /noise_effect_gate_disabled/);
});

function records() {
  return Array.from({ length: 14 }, (_, i) => ({ phase: 101 + i, stage: 2, integrity: true,
    stack_span_kind: 'configured', configured_main_stack_bytes: 16384, stack_low_water: 2048,
    stack_pointer_inside: true, internal_free: 10000, internal_largest: 5000 }));
}
function events() {
  return [{ event: 'boot', execution_profile: 'virtual-ultra205', compiled_source_sha256: source, heartbeat_cutoff_ms: 2800 },
    { event: 'noise_probe', result: { schema: 'bitaxe-noise-probe-v1', seed: 1, authenticated: true,
      frame_round_trip: true, resources_released: true, payload_bytes: 32, hardware_qualified: false },
    minimum_main_stack_free_bytes: 2048, required_margin_bytes: 2048, configured_main_stack_bytes: 16384 }];
}
const processResult = { released: true, interrupted: false, timedOut: true, code: null };
function judgment(input = events(), checkpoints = records(), outcome = processResult, panic = false) {
  return judgeNoiseProbe(input, checkpoints, { compiled_source_sha256: source }, outcome, panic);
}

test('collection cutoff is separate from crypto completion and measured unchanged margin', () => {
  // Arrange / Act
  const checks = judgment();
  // Assert
  assert.equal(checks.every(check => check.status === 'passed'), true);
});

for (const [name, mutate, checkId] of [
  ['missing result', input => input.pop(), 'noise_authenticated_round_trip'],
  ['claimed auth without frame', input => { input[1].result.frame_round_trip = false; }, 'noise_authenticated_round_trip'],
  ['unreleased ciphers', input => { input[1].result.resources_released = false; }, 'noise_resources_released'],
  ['stale compiled identity', input => { input[0].compiled_source_sha256 = 'c'.repeat(64); }, 'boot_identity'],
  ['missing actual margin', input => { delete input[1].minimum_main_stack_free_bytes; }, 'noise_stack_margin'],
  ['increased main stack', input => { input[1].configured_main_stack_bytes = 32768; }, 'noise_stack_margin'],
  ['duplicate completion', input => input.push(input[1]), 'noise_authenticated_round_trip'],
]) {
  test(`result judge rejects ${name}`, () => {
    // Arrange
    const input = events(); mutate(input);
    // Act
    const checks = judgment(input);
    // Assert
    assert.equal(checks.find(check => check.id === checkId).status, 'failed');
  });
}

test('panic remains failure even with successful release and fabricated completion facts', () => {
  // Arrange / Act
  const checks = judgment(events(), records(), processResult, true);
  // Assert
  assert.equal(checks.find(check => check.id === 'no_unexpected_panic').status, 'failed');
});

test('failed natural process completion is not credited by resource release', () => {
  // Arrange / Act
  const checks = judgment(events(), records(), { ...processResult, timedOut: false, code: 7 });
  // Assert
  assert.equal(checks.find(check => check.id === 'emulator_released').status, 'failed');
});

test('partial stage1 records preserve stack facts and withhold heap proof', () => {
  // Arrange
  const raw = Buffer.alloc(768);
  [0x564e5031, 110, 1, 0, 0, 1000, 16384, 900, 0, 0, 0, 0].forEach((value, i) => raw.writeUInt32LE(value, i * 4));
  // Act
  const result = parseNoiseCheckpoints(raw);
  // Assert
  assert.equal(result[0].phase, 110); assert.equal(result[0].integrity, null);
  assert.equal(result[0].stack_pointer_inside, false); assert.equal(result[0].stack_low_water, 0);
  assert.equal(judgment(events(), result).find(check => check.id === 'noise_phase_integrity').status, 'failed');
});

test('finalization refuses a real live writer in an evidence ancestor', async () => {
  // Arrange
  const repo = await mkdtemp(join(tmpdir(), 'noise-writer-')), parent = join(repo, 'evidence'), root = join(parent, 'attempt');
  await mkdir(root, { recursive: true, mode: 0o700 });
  await writeFile(join(parent, 'qemu.writer.json'), '{}', { flag: 'wx', mode: 0o600 });
  try {
    // Act / Assert
    await assert.rejects(requireNoNoiseWriters(root, repo), /noise_live_writer/);
  } finally { await rm(repo, { recursive: true }); }
});

function elfHeader(bytes) {
  bytes.writeUInt32BE(0x7f454c46); bytes[4] = 1; bytes[5] = 1; bytes.writeUInt16LE(94, 18);
}
test('binary core extraction resolves only the exact new journal symbol', () => {
  // Arrange
  const elf = Buffer.alloc(256), core = Buffer.alloc(852); elfHeader(elf); elfHeader(core);
  elf.writeUInt32LE(64, 32); elf.writeUInt16LE(40, 46); elf.writeUInt16LE(3, 48);
  elf.writeUInt32LE(2, 108); elf.writeUInt32LE(184, 120); elf.writeUInt32LE(16, 124); elf.writeUInt32LE(2, 128); elf.writeUInt32LE(16, 140);
  const name = Buffer.from('\0BITAXE_VIRTUAL_NOISE_CHECKPOINTS\0');
  elf.writeUInt32LE(3, 148); elf.writeUInt32LE(200, 160); elf.writeUInt32LE(name.length, 164); name.copy(elf, 200);
  elf.writeUInt32LE(1, 184); elf.writeUInt32LE(0x3fc90000, 188); elf.writeUInt32LE(768, 192);
  core.writeUInt32LE(52, 28); core.writeUInt16LE(32, 42); core.writeUInt16LE(1, 44);
  core.writeUInt32LE(1, 52); core.writeUInt32LE(84, 56); core.writeUInt32LE(0x3fc90000, 60); core.writeUInt32LE(768, 68);
  core.writeUInt32LE(0x564e5031, 84); core.writeUInt32LE(101, 88); core.writeUInt32LE(1, 92);
  // Act
  const result = extractNoiseCheckpoints(elf, core);
  // Assert
  assert.equal(result.length, 1); assert.equal(result[0].phase, 101);
  elf.writeUInt32LE(767, 192);
  assert.throws(() => extractNoiseCheckpoints(elf, core), /noise_checkpoint_symbol_missing/);
});


for (const [name, prepare, original] of [
  ['failed result persistence', async root => writeFile(join(root, 'noise-result.json'), 'sealed-retained', { flag: 'wx' }), null],
  ['earliest panic plus cleanup failure', async root => writeFile(join(root, 'qemu.writer.json'), '{}', { flag: 'wx' }), { phase: 'noise', category: 'unexpected_target_panic' }],
]) {
  test(`finalization preserves truthful failure for ${name}`, async () => {
    // Arrange
    const repo = await mkdtemp(join(tmpdir(), 'noise-finalize-')), root = join(repo, 'attempt');
    await mkdir(root, { mode: 0o700 }); await prepare(root);
    const result = { status: 'failed', checks: judgment(), maybe_earliest_failure: original, cleanup_failures: [] };
    try {
      // Act / Assert
      await assert.rejects(finalizeNoiseResult(result, root, repo, async () => {}));
      assert.equal(result.status, 'failed'); assert.equal(result.cleanup_failures.length, 1);
      if (original) assert.deepEqual(result.maybe_earliest_failure, original);
      else assert.equal(await readFile(join(root, 'noise-result.json'), 'utf8'), 'sealed-retained');
    } finally { await rm(repo, { recursive: true }); }
  });
}
