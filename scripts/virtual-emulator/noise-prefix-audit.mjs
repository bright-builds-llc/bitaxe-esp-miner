import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile, stat } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseNoiseFrames, measureNoisePath, nestedCryptoPath, validateNativeNoiseAudit } from './noise-stack-audit.mjs';
import { sourceSnapshot } from './identity.mjs';
import { nativeElfSymbols } from './elf-symbols.mjs';

export const PREFIX_STOPS = [101, 102, 103, 105, 106, 107, 109, 110];
const directory = dirname(fileURLToPath(import.meta.url));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const PREFIX = ['bitaxe_virtual_firmware::main', 'bitaxe_virtual_firmware::guest::run',
  'bitaxe_virtual_firmware::noise_probe::run_prefix_and_emit', 'bitaxe_simulation::noise_probe::prefix::run_until'];
const HANDSHAKE = 'bitaxe_simulation::noise_probe::prefix::handshake_until_prefix';
const CONFIG = ['CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384', 'CONFIG_FREERTOS_CHECK_STACKOVERFLOW_CANARY=y',
  'CONFIG_FREERTOS_TASK_FUNCTION_WRAPPER=y', '# CONFIG_LIBC_NEWLIB_NANO_FORMAT is not set'];

/** Compiler-folded marker names are admitted only from the actual ELF symbol table. */
function markerAliases(elf, frames) {
  const aliases = [];
  for (const symbol of nativeElfSymbols(elf)) {
    const { name, address } = symbol;
    if (!/^bitaxe_virtual_noise_(?:prefix_cutoff|prefix_released|checkpoint_complete)$/.test(name) || frames.byName.has(name)) continue;
    const native = frames.byAddress.get(address);
    if (!native || symbol.type !== 2 || !symbol.defined) throw Error("noise_prefix_audit_alias_target");
    frames.byName.set(name, new Set([address]));
    aliases.push({ symbol: name, canonical_native_symbol: native.name, adds_native_frame: false });
  }
  return aliases;
}

export async function prefixNoiseAuditIdentity() {
  const hash = createHash('sha256');
  for (const name of ['noise-prefix-audit.mjs', 'noise-stack-audit.mjs', 'identity.mjs', 'elf-symbols.mjs']) {
    hash.update(name); hash.update('\0'); hash.update(await readFile(join(directory, name))); hash.update('\0');
  }
  return hash.digest('hex');
}

/** Runtime selectors share this campaign image; a different helper is not a baseline fix. */
export function auditNoisePrefixStack(disassembly, config, selectedStop, maybeElf) {
  if (!PREFIX_STOPS.includes(selectedStop) || !CONFIG.every(line => config.split('\n').includes(line)) ||
      config.split('\n').filter(line => line.startsWith('CONFIG_ESP_MAIN_TASK_STACK_SIZE=')).length !== 1) throw Error('noise_prefix_audit_scope');
  const frames = parseNoiseFrames(disassembly), names = [...PREFIX, HANDSHAKE];
  const aliases = maybeElf ? markerAliases(maybeElf, frames) : [];
  const required = [{ id: 'selected_prefix', names }];
  if (selectedStop >= 103) required.push({ id: 'initiator_constructor', names: [...names,
    'bitaxe_stratum::v2::noise::NoiseInitiator::new', 'noise_sv2::initiator::Initiator::new_with_rng'] });
  if (selectedStop >= 105) required.push({ id: 'act_one', names: [...names, 'bitaxe_stratum::v2::noise::NoiseInitiator::act_one'] });
  if (selectedStop >= 107) required.push({ id: 'responder_keypair_constructor', names: [...names,
    'rustsecp256k1_v0_9_2_keypair_create'] });
  if (selectedStop >= 109) {
    required.push({ id: 'responder_ecdh', names: [...names, 'secp256k1::ellswift::ElligatorSwift::shared_secret',
      'rustsecp256k1_v0_9_2_ellswift_xdh', 'rustsecp256k1_v0_9_2_ecmult_const$part$0', 'rustsecp256k1_v0_9_2_gej_add_ge'] });
    required.push({ id: 'responder_sign', names: [...names,
      'secp256k1::schnorr::<impl secp256k1::Secp256k1<C>>::sign_schnorr_with_rng',
      'rustsecp256k1_v0_9_2_schnorrsig_sign_internal'] });
  }
  const paths = required.map(({ id, names: path }) => {
    if (id === 'selected_prefix') return { id, ...measureNoisePath(frames, path), nested_crypto_scope: false };
    const nested = nestedCryptoPath(frames, path.at(-1));
    return { id, ...measureNoisePath(frames, [...path.slice(0, -1), ...nested.names]), nested_crypto_scope: true,
      crypto_cycle_count: nested.cycle_count, outside_crypto_family_edges: nested.outside_crypto_family_edges,
      complete_callgraph_bound: false };
  });
  const maybeHandshake = frames.byName.get(HANDSHAKE);
  if (!maybeHandshake || maybeHandshake.size !== 1) throw Error('noise_prefix_audit_frame');
  const handshakeFrame = frames.byAddress.get([...maybeHandshake][0]).bytes;
  const hooks = ['bitaxe_virtual_noise_checkpoint', 'bitaxe_virtual_noise_prefix_cutoff',
    'bitaxe_virtual_noise_checkpoint_complete', 'bitaxe_virtual_noise_prefix_released'].map(name => {
    const variants = frames.byName.get(name);
    if (!variants || variants.size !== 1) throw Error('noise_prefix_audit_hook');
    const frame = frames.byAddress.get([...variants][0]);
    if (frame.bytes === null) throw Error('noise_prefix_audit_hook');
    return { symbol: name, native_frame_bytes: frame.bytes };
  });
  return { schema: 'bitaxe-noise-prefix-native-audit-v2', profile: 'noise-prefix-valid-seed1',
    selected_stop: selectedStop, selector_mode: 'runtime_campaign_same_elf', main_stack_bytes: 16384,
    required_margin_bytes: 2048, sdk_extra_stack_bytes_report_only: 512, available_selected_path_bytes: 14336,
    actual_prefix_handshake_frame_bytes: handshakeFrame, baseline_reference_handshake_frame_bytes: 6368,
    baseline_helper_changed: true, prefix_frame_delta_from_baseline_bytes: handshakeFrame - 6368,
    selected_path_budget_fit: paths.every(path => path.frame_bytes <= 14336), paths, measurement_hooks: hooks,
    folded_marker_aliases: aliases,
    measurement_callback_edge: 'source_bound_indirect_callback_requires_live_snapshot',
    complete_callgraph_bound: false, responder_constructor_inlined_coverage: 'caller_frame_plus_keypair_native_boundary',
    runtime_stack_and_heap_proof_required: true, full_noise_eligible: false,
    excluded_certificate_branch: { minimum_known_native_path_bytes: 17264, phase: 111,
      reason: 'completion_outside_prefix_scope_and_baseline_certificate_path_exceeds_stack' } };
}

export async function validatePrefixNoiseAudit(receipt, expected, { disassembly, sdkconfig, elf }, selectedStop) {
  if (receipt?.schema !== 'bitaxe-noise-prefix-native-audit-v2' || receipt.profile !== 'noise-prefix-valid-seed1' ||
      receipt.selected_stop !== selectedStop || receipt.full_noise_eligible !== false ||
      receipt.bindings?.auditor_sha256 !== await prefixNoiseAuditIdentity() ||
      receipt.bindings.elf_sha256 !== expected.elfSha256 || receipt.bindings.sdkconfig_sha256 !== expected.sdkconfigSha256 ||
      receipt.bindings.compiled_source_sha256 !== expected.compiledSourceSha256 ||
      digest(disassembly) !== receipt.bindings.disassembly_sha256 || digest(sdkconfig) !== expected.sdkconfigSha256) throw Error('noise_prefix_audit_binding');
  if (receipt.folded_marker_aliases?.length && (!Buffer.isBuffer(elf) || digest(elf) !== expected.elfSha256)) throw Error('noise_prefix_audit_alias_elf');
  const measured = auditNoisePrefixStack(disassembly, sdkconfig, selectedStop, elf);
  for (const [key, value] of Object.entries(measured)) {
    if (JSON.stringify(receipt[key]) !== JSON.stringify(value)) throw Error('noise_prefix_audit_native_proof');
  }
  if (!measured.selected_path_budget_fit) throw Error('noise_prefix_audit_budget');
  return receipt;
}

export async function main(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    if (!['--elf', '--sdkconfig', '--compiled-source-sha256', '--cutoff-phase', '--output'].includes(key) ||
        Object.hasOwn(args, key) || typeof argv[index + 1] !== 'string') throw Error('noise_prefix_audit_arguments');
    args[key] = argv[index + 1];
  }
  if (Object.keys(args).length !== 5 || !/^[a-f0-9]{64}$/.test(args['--compiled-source-sha256'])) throw Error('noise_prefix_audit_arguments');
  process.umask(0o077);
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const output = resolve(repo, args['--output']), elfPath = resolve(repo, args['--elf']), configPath = resolve(repo, args['--sdkconfig']);
  if (!output.startsWith(`${repo}/`) || ((await stat(dirname(output))).mode & 0o777) !== 0o700) throw Error('noise_prefix_audit_private_root');
  execFileSync('git', ['check-ignore', '--quiet', '--', output], { cwd: repo });
  const source = await sourceSnapshot(repo), elf = await readFile(elfPath), config = await readFile(configPath);
  if (source.sha256 !== args['--compiled-source-sha256'] || !elf.includes(Buffer.from(source.sha256)) ||
      elf.length < 52 || elf.readUInt32LE(0) !== 0x464c457f || elf.readUInt16LE(18) !== 94 ||
      !elf.includes(Buffer.from('BITAXE_EXECUTION_PROFILE=virtual-ultra205'))) throw Error('noise_prefix_audit_source');
  const version = 'esp-14.2.0_20260121';
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json'), 'utf8'));
  if (!manifest.tools.find(tool => tool.name === 'xtensa-esp-elf')?.versions.some(item => item.name === version && item.status === 'recommended')) throw Error('noise_prefix_audit_tool');
  const tool = join(repo, '.embuild/espressif/tools/xtensa-esp-elf', version, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump');
  const disassembly = execFileSync(tool, ['-d', '-C', elfPath], { encoding: 'utf8', timeout: 120000,
    maxBuffer: 100 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] }).replace(/^.+:\s+file format\s+/m, 'NATIVE_ELF: file format ');
  await writeFile(`${output}.disassembly.private`, disassembly, { flag: 'wx', mode: 0o600 });
  const receipt = auditNoisePrefixStack(disassembly, config.toString(), Number(args['--cutoff-phase']), elf);
  receipt.bindings = { elf_sha256: digest(elf), sdkconfig_sha256: digest(config), compiled_source_sha256: source.sha256,
    auditor_sha256: await prefixNoiseAuditIdentity(), objdump_version: version, objdump_sha256: digest(await readFile(tool)), disassembly_sha256: digest(disassembly) };
  await validateNativeNoiseAudit(receipt, elfPath, repo);
  await writeFile(output, JSON.stringify(receipt, null, 2), { flag: 'wx', mode: 0o600 });
  return { status: receipt.selected_path_budget_fit ? 'passed' : 'blocked', selected_stop: receipt.selected_stop, full_noise_eligible: false };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => { console.log(JSON.stringify(result)); if (result.status !== 'passed') process.exitCode = 1; })
    .catch(() => { console.error('{"status":"blocked","category":"noise_prefix_native_audit_failed"}'); process.exitCode = 1; });
}
