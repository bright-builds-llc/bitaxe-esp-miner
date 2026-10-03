import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile, stat, lstat } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { sourceSnapshot } from './identity.mjs';

const PROFILE = 'noise-only-valid-seed1';
const MAIN = 16384;
const MARGIN = 2048;
// Matches `bitaxe_simulation::HANDSHAKE_STACK_BYTES`; helper paths share the main budget.
const HELPER_STACK = 16384;
const OBJDUMP_VERSION = 'esp-14.2.0_20260121';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const hexDigest = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const moduleDirectory = dirname(fileURLToPath(import.meta.url));

/** Word addresses that some `l32r` loads; their bytes are data, whatever objdump decodes them as. */
export function literalWords(disassembly) {
  const words = new Set();
  for (const match of disassembly.matchAll(/\sl32r\s+a\d+,\s*([a-f0-9]+)\s/g)) words.add(Number.parseInt(match[1], 16));
  return words;
}

function insideLiteral(address, bytes, literals) {
  for (let offset = 0; offset < bytes; offset++) if (literals.has((address + offset) & ~3)) return true;
  return false;
}

function nativeCalls(body, literals = new Set()) {
  const calls = [], registers = new Map();
  let indirectCalls = 0;
  for (const line of body.split('\n')) {
    const maybeInstruction = line.match(/^\s*([a-f0-9]+):\s+([a-f0-9][a-f0-9 ]*?)\s+([a-z][a-z0-9.]*)\s*(.*)$/);
    if (!maybeInstruction) continue;
    const [, at, encoding, op, operands] = maybeInstruction;
    // Literal pools between functions can decode as calls; they are never executed.
    if (insideLiteral(Number.parseInt(at, 16), encoding.replaceAll(' ', '').length / 2, literals)) { registers.clear(); continue; }
    const maybeDirect = operands.match(/^([a-f0-9]+)\s+</);
    if (/^call(?:0|4|8|12)$/.test(op) && maybeDirect) {
      calls.push(Number.parseInt(maybeDirect[1], 16)); registers.clear(); continue;
    }
    if (/^callx(?:0|4|8|12)$/.test(op)) {
      const maybeTarget = registers.get(operands.trim());
      if (maybeTarget === undefined) indirectCalls++; else calls.push(maybeTarget);
      registers.clear(); continue;
    }
    const maybeDestination = operands.match(/^(a\d+)(?:,|$)/)?.[1];
    if (op === 'l32r') {
      const maybeLiteral = operands.match(/\(([a-f0-9]+) <.+>\)\s*$/);
      if (maybeLiteral) { registers.set(maybeDestination, Number.parseInt(maybeLiteral[1], 16)); continue; }
    }
    if (/^mov(?:\.n)?$/.test(op)) {
      const maybeSource = registers.get(operands.split(',')[1]?.trim());
      if (maybeSource !== undefined) { registers.set(maybeDestination, maybeSource); continue; }
    }
    if (/^(?:b|j|ret|loop)/.test(op)) registers.clear();
    // Stores read their first operand; arithmetic and loads replace it.
    else if (maybeDestination && !/^s(?:8|16|32)i/.test(op)) registers.delete(maybeDestination);
  }
  return { calls, indirectCalls };
}

/** Native entries are physical frames; aliases and inline DWARF frames add no frame. */
export function parseNoiseFrames(disassembly) {
  const byAddress = new Map(), byName = new Map();
  const labels = [...disassembly.matchAll(/^([a-f0-9]+) <(.+)>:\s*$/gm)], literals = literalWords(disassembly);
  for (let index = 0; index < labels.length; index++) {
    const label = labels[index], address = Number.parseInt(label[1], 16);
    const body = disassembly.slice(label.index + label[0].length, labels[index + 1]?.index ?? disassembly.length);
    const maybeEntry = body.match(/^\s*[a-f0-9]+:\s+[a-f0-9 ]+\s+entry\s+a1,\s*(0x[a-f0-9]+|[0-9]+)/m);
    const observed = nativeCalls(body, literals);
    const frame = byAddress.get(address) ?? { name: label[2], bytes: maybeEntry ? Number(maybeEntry[1]) : null,
      calls: [], indirectCalls: 0, aliases: [] };
    frame.aliases.push(label[2]); frame.calls.push(...observed.calls);
    frame.indirectCalls += observed.indirectCalls;
    byAddress.set(address, frame);
    const variants = byName.get(label[2]) ?? new Set(); variants.add(address); byName.set(label[2], variants);
  }
  return { byAddress, byName };
}

function exactFrame(frames, name) {
  const maybeAddresses = frames.byName.get(name);
  if (maybeAddresses === undefined) throw Error('noise_stack_required_symbol_missing');
  return [...maybeAddresses];
}

/** Find a real direct-call path, preserving unknown frames rather than crediting zero. */
function pathBetween(frames, start, end) {
  const queue = [[start]], seen = new Set([start]);
  for (let index = 0; index < queue.length && index < 20000; index++) {
    const path = queue[index], last = path.at(-1);
    if (last === end) return path;
    if (path.length >= 128) continue;
    for (const target of frames.byAddress.get(last)?.calls ?? []) {
      if (seen.has(target)) continue;
      seen.add(target); queue.push([...path, target]);
    }
  }
  return null;
}

/** Closed named paths establish lower bounds, never a global callgraph upper bound. */
export function measureNoisePath(frames, names) {
  const addresses = names.map(name => exactFrame(frames, name));
  let paths = addresses[0].map(address => [address]);
  for (let index = 1; index < addresses.length; index++) {
    paths = paths.flatMap(path => addresses[index].flatMap(target => {
      const maybePart = pathBetween(frames, path.at(-1), target);
      return maybePart ? [[...path, ...maybePart.slice(1)]] : [];
    }));
    if (!paths.length || paths.length > 64) throw Error('noise_stack_required_call_missing');
  }
  const results = paths.map(path => {
    const native = [...new Set(path)].map(address => frames.byAddress.get(address));
    if (native.some(frame => frame.bytes === null)) throw Error('noise_stack_required_entry_missing');
    return { native_symbols: native.map(frame => frame.name), frame_bytes: native.reduce((sum, frame) => sum + frame.bytes, 0),
      native_frame_count: native.length, unresolved_call_edges: native.reduce((sum, frame) => sum + frame.indirectCalls, 0),
      selected_symbol_variants: paths.length };
  });
  return results.reduce((largest, result) => result.frame_bytes > largest.frame_bytes ? result : largest);
}

const CRYPTO_FAMILY = /^(?:noise_sv2::|bitaxe_stratum::v2::noise::|secp256k1::|rustsecp256k1_v0_9_2_|sha2::|hmac::|chacha20poly1305::|chacha20::|poly1305::)/;
const frameSum = (frames, path) => path.reduce((sum, address) => sum + frames.byAddress.get(address).bytes, 0);

/** Longest resolved crypto-family path, retaining external-edge coverage gaps. */
export function nestedCryptoPath(frames, boundaryName) {
  const starts = frames.byName.get(boundaryName);
  if (!starts) throw Error('noise_crypto_boundary_missing');
  const memo = new Map(), cycles = new Set(), gaps = new Set();
  function walk(address, active = new Set()) {
    if (active.has(address)) { cycles.add(address); return []; }
    if (memo.has(address)) return memo.get(address);
    const frame = frames.byAddress.get(address);
    if (!frame || frame.bytes === null) throw Error('noise_crypto_entry_missing');
    let best = [address];
    const nextActive = new Set(active).add(address);
    for (const target of frame.calls) {
      const child = frames.byAddress.get(target);
      if (!child || !CRYPTO_FAMILY.test(child.name)) { gaps.add(target); continue; }
      const candidate = [address, ...walk(target, nextActive)];
      if (frameSum(frames, candidate) > frameSum(frames, best)) best = candidate;
    }
    memo.set(address, best); return best;
  }
  const largest = [...starts].map(address => walk(address))
    .reduce((best, path) => frameSum(frames, path) > frameSum(frames, best) ? path : best);
  return { names: largest.map(address => frames.byAddress.get(address).name),
    cycle_count: cycles.size, outside_crypto_family_edges: gaps.size, complete_callgraph_bound: false };
}

const PREFIX = ['bitaxe_virtual_firmware::main', 'bitaxe_virtual_firmware::guest::run',
  'bitaxe_virtual_firmware::noise_probe::run_and_emit', 'bitaxe_simulation::noise_probe::run',
  'bitaxe_simulation::noise_probe::handshake_and_frame'];
const INITIATOR = [...PREFIX, 'bitaxe_simulation::noise_probe::prepare_initiator'];
const RESPONDER = [...PREFIX, 'bitaxe_simulation::noise_probe::respond'];
const COMPLETE = 'bitaxe_stratum::v2::noise::completion::<impl bitaxe_stratum::v2::noise::NoiseInitiator>::complete_diagnostic_into';
const COMPLETION = [...PREFIX, COMPLETE];
const CONSTRUCT_RESPONDER = 'bitaxe_simulation::v2::exchange::construct_responder';
const STEP_RESPONDER = 'bitaxe_simulation::v2::exchange::step_responder';
const STEP_TWO = 'noise_sv2::initiator::Initiator::step_2_with_now';
const VERIFY = 'rustsecp256k1_v0_9_2_schnorrsig_verify';
// The composed Start path runs its handshake on a dedicated helper thread.
const HELPER = ['bitaxe_simulation::v2::exchange::handshake'];
const CONTROL_EXCHANGE = 'bitaxe_simulation::v2::exchange::Exchange::new';
// Each crypto boundary extends its caller chain by the longest resolved crypto-family descent.
const REQUIRED_PATHS = [
  { id: 'initiator_constructor', names: INITIATOR, crypto: 'noise_sv2::initiator::Initiator::new_with_rng' },
  { id: 'act_one', names: INITIATOR, crypto: 'bitaxe_stratum::v2::noise::NoiseInitiator::act_one' },
  // Our non-inlined helpers are the boundaries; the library calls may inline into them.
  { id: 'responder_constructor', names: RESPONDER, crypto: CONSTRUCT_RESPONDER },
  { id: 'responder_ecdh_and_sign', names: RESPONDER, crypto: STEP_RESPONDER },
  { id: 'completion', names: COMPLETION, crypto: STEP_TWO },
  // The valid certificate branch was previously omitted; it must be bounded by name.
  { id: 'certificate_verification', names: [...COMPLETION, STEP_TWO], crypto: VERIFY },
  { id: 'encrypted_frame', names: PREFIX, crypto: 'bitaxe_simulation::noise_probe::frame_round_trip' },
  { id: 'outcome_emission', names: [...PREFIX.slice(0, 3), 'bitaxe_virtual_firmware::noise_probe::emit_outcome'], crypto: null },
  { id: 'composed_helper_initiator', names: [...HELPER, 'bitaxe_simulation::v2::exchange::prepare_initiator'],
    crypto: 'noise_sv2::initiator::Initiator::new_with_rng' },
  { id: 'composed_helper_responder_constructor', names: HELPER, crypto: CONSTRUCT_RESPONDER },
  { id: 'composed_helper_responder_step', names: HELPER, crypto: STEP_RESPONDER },
  { id: 'composed_helper_completion', names: [...HELPER, COMPLETE], crypto: STEP_TWO },
  { id: 'composed_helper_certificate', names: [...HELPER, COMPLETE, STEP_TWO], crypto: VERIFY },
];
// Crypto entry points that must never be reached on the control stack.
const CONTROL_FORBIDDEN = [HELPER[0], COMPLETE, STEP_TWO, STEP_RESPONDER, CONSTRUCT_RESPONDER];

/** True only when no direct-call path leads from the control-stack exchange into crypto. */
export function controlStackIsolated(frames) {
  const starts = frames.byName.get(CONTROL_EXCHANGE);
  if (!starts) throw Error('noise_stack_required_symbol_missing');
  return CONTROL_FORBIDDEN.every(name => [...(frames.byName.get(name) ?? [])]
    .every(target => [...starts].every(start => pathBetween(frames, start, target) === null)));
}

function measureRequiredPath(frames, { id, names, crypto }) {
  if (crypto === null) return { id, ...measureNoisePath(frames, names), crypto_boundary: null };
  const nested = nestedCryptoPath(frames, crypto);
  return { id, ...measureNoisePath(frames, [...names, ...nested.names]), crypto_boundary: crypto,
    crypto_cycle_count: nested.cycle_count, outside_crypto_family_edges: nested.outside_crypto_family_edges };
}

/** Audit only the selected synthetic valid probe; source branches are not runtime observations. */
export function auditNoiseStack(disassembly, config) {
  const lines = config.split('\n');
  if (lines.filter(line => line.startsWith('CONFIG_ESP_MAIN_TASK_STACK_SIZE=')).length !== 1 ||
      !lines.includes(`CONFIG_ESP_MAIN_TASK_STACK_SIZE=${MAIN}`) ||
      !lines.includes('CONFIG_FREERTOS_CHECK_STACKOVERFLOW_CANARY=y') ||
      !lines.includes('CONFIG_FREERTOS_TASK_FUNCTION_WRAPPER=y') ||
      !lines.includes('# CONFIG_LIBC_NEWLIB_NANO_FORMAT is not set')) throw Error('noise_stack_config');
  const frames = parseNoiseFrames(disassembly);
  const paths = REQUIRED_PATHS.map(required => {
    const path = measureRequiredPath(frames, required);
    return { ...path, fit: path.frame_bytes <= MAIN - MARGIN };
  });
  const isolated = controlStackIsolated(frames);
  return { schema: 'bitaxe-noise-stack-audit-v3', profile: PROFILE, main_stack_bytes: MAIN,
    helper_stack_bytes: HELPER_STACK, sdk_extra_stack_bytes: 512, required_margin_bytes: MARGIN,
    available_selected_path_bytes: MAIN - MARGIN, composed_crypto_isolated_from_control_stack: isolated,
    helper_thread_entry_frames_bounded: false,
    selected_path_budget_fit: isolated && paths.every(path => path.fit), paths, complete_callgraph_bound: false,
    scope: 'named_native_paths_with_longest_resolved_crypto_descent', runtime_high_water_required_bytes: MARGIN,
    unbounded_edges: ['indirect_calls', 'outside_crypto_family_calls', 'crypto_cycles', 'drop_glue'] };
}

/** Identity covers this auditor and the source binding helper it imports. */
export async function noiseAuditIdentity() {
  const hash = createHash('sha256');
  for (const name of ['noise-stack-audit.mjs', 'identity.mjs']) {
    hash.update(name); hash.update('\0'); hash.update(await readFile(join(moduleDirectory, name))); hash.update('\0');
  }
  return hash.digest('hex');
}

async function pinnedObjdump(repo) {
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json'), 'utf8'));
  if (!manifest.tools.find(tool => tool.name === 'xtensa-esp-elf')?.versions.some(version => version.name === OBJDUMP_VERSION && version.status === 'recommended')) throw Error('noise_audit_objdump_pin');
  return join(repo, '.embuild/espressif/tools/xtensa-esp-elf', OBJDUMP_VERSION, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump');
}

function disassemble(tool, elf) {
  const output = execFileSync(tool, ['-d', '-C', resolve(elf)], { encoding: 'utf8', timeout: 120000,
    maxBuffer: 100 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
  // Exact ELF identity is bound separately; moving the identical ELF must not alter its proof.
  return output.replace(/^.+:\s+file format\s+/m, 'NATIVE_ELF: file format ');
}

/** Re-run the pinned offline tool so an edited report cannot relabel another native image. */
export async function validateNativeNoiseAudit(receipt, elf, repo) {
  const tool = await pinnedObjdump(repo);
  if (digest(await readFile(elf)) !== receipt.bindings?.elf_sha256 ||
      digest(await readFile(tool)) !== receipt.bindings?.objdump_sha256 ||
      receipt.bindings?.objdump_version !== OBJDUMP_VERSION ||
      digest(disassemble(tool, elf)) !== receipt.bindings?.disassembly_sha256) throw Error('noise_audit_native_proof');
}

/** An artifact match admits selected paths only; runtime margin and heap proof remain required. */
export async function validateNoiseAudit(receipt, { elfSha256, sdkconfigSha256, compiledSourceSha256 }, { disassembly, sdkconfig } = {}) {
  if (receipt?.schema !== 'bitaxe-noise-stack-audit-v3' || receipt.profile !== PROFILE ||
      receipt.composed_crypto_isolated_from_control_stack !== true || receipt.helper_stack_bytes !== HELPER_STACK ||
      receipt.selected_path_budget_fit !== true || receipt.complete_callgraph_bound !== false ||
      receipt.main_stack_bytes !== MAIN || receipt.required_margin_bytes !== MARGIN ||
      receipt.available_selected_path_bytes !== MAIN - MARGIN || receipt.sdk_extra_stack_bytes !== 512 ||
      receipt.runtime_high_water_required_bytes !== MARGIN ||
      receipt.scope !== 'named_native_paths_with_longest_resolved_crypto_descent' ||
      receipt.bindings?.auditor_sha256 !== await noiseAuditIdentity()) throw Error('noise_audit_invalid');
  const expected = { elf_sha256: elfSha256, sdkconfig_sha256: sdkconfigSha256, compiled_source_sha256: compiledSourceSha256 };
  for (const [key, value] of Object.entries(expected)) {
    if (!hexDigest(value) || receipt.bindings[key] !== value) throw Error('noise_audit_binding');
  }
  if (receipt.bindings.objdump_version !== OBJDUMP_VERSION || !hexDigest(receipt.bindings.objdump_sha256) ||
      !Array.isArray(receipt.paths) || receipt.paths.length !== REQUIRED_PATHS.length) throw Error('noise_audit_invalid');
  for (const required of REQUIRED_PATHS) {
    const paths = receipt.paths.filter(path => path.id === required.id);
    if (paths.length !== 1 || paths[0].fit !== true || !Number.isSafeInteger(paths[0].frame_bytes) ||
        paths[0].frame_bytes < 1 || paths[0].frame_bytes > MAIN - MARGIN ||
        !Array.isArray(paths[0].native_symbols) || paths[0].crypto_boundary !== required.crypto ||
        [...required.names, ...(required.crypto ? [required.crypto] : [])].some(name => !paths[0].native_symbols.includes(name))) {
      throw Error('noise_audit_invalid');
    }
  }
  if (typeof disassembly !== 'string' || typeof sdkconfig !== 'string' ||
      !hexDigest(receipt.bindings.disassembly_sha256) || digest(disassembly) !== receipt.bindings.disassembly_sha256 ||
      digest(sdkconfig) !== sdkconfigSha256) throw Error('noise_audit_native_proof');
  const measured = auditNoiseStack(disassembly, sdkconfig);
  if (!measured.selected_path_budget_fit || JSON.stringify(measured.paths) !== JSON.stringify(receipt.paths)) throw Error('noise_audit_native_proof');
  return receipt;
}

/** Offline native audit: pinned installed objdump only; never installs or opens hardware. */
export async function main(argv) {
  process.umask(0o077);
  const options = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    if (!['--elf', '--sdkconfig', '--compiled-source-sha256', '--output'].includes(key) ||
        Object.hasOwn(options, key) || typeof argv[index + 1] !== 'string') throw Error('noise_audit_arguments');
    options[key] = argv[index + 1];
  }
  if (Object.keys(options).length !== 4 || !hexDigest(options['--compiled-source-sha256'])) throw Error('noise_audit_arguments');
  const repo = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const output = resolve(options['--output']), parent = dirname(output);
  if (!output.startsWith(`${repo}/`) || ((await stat(parent)).mode & 0o777) !== 0o700 || (await lstat(parent)).isSymbolicLink()) throw Error('noise_audit_private_root');
  execFileSync('git', ['check-ignore', '--quiet', '--', output], { cwd: repo });
  const elf = await readFile(options['--elf']), config = await readFile(options['--sdkconfig']);
  const source = await sourceSnapshot(repo);
  if (source.sha256 !== options['--compiled-source-sha256'] || !elf.includes(Buffer.from(source.sha256)) ||
      !elf.includes(Buffer.from('BITAXE_EXECUTION_PROFILE=virtual-ultra205')) ||
      elf.readUInt32LE(0) !== 0x464c457f || elf[4] !== 1 || elf[5] !== 1 || elf.readUInt16LE(18) !== 94) throw Error('noise_audit_native_binding');
  const tool = await pinnedObjdump(repo);
  const disassembly = disassemble(tool, options['--elf']);
  await writeFile(`${output}.disassembly.private`, disassembly, { flag: 'wx', mode: 0o600 });
  const receipt = auditNoiseStack(disassembly, config.toString());
  receipt.bindings = { elf_sha256: digest(elf), sdkconfig_sha256: digest(config), compiled_source_sha256: source.sha256,
    auditor_sha256: await noiseAuditIdentity(), objdump_version: OBJDUMP_VERSION, objdump_sha256: digest(await readFile(tool)),
    disassembly_sha256: digest(disassembly) };
  await writeFile(output, JSON.stringify(receipt, null, 2), { flag: 'wx', mode: 0o600 });
  return { status: receipt.selected_path_budget_fit ? 'passed' : 'blocked', profile: PROFILE,
    category: receipt.selected_path_budget_fit ? 'selected_paths_fit' : 'selected_native_path_exceeds_budget',
    selected_path_budget_fit: receipt.selected_path_budget_fit, complete_callgraph_bound: false };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => { console.log(JSON.stringify(result)); if (result.status !== 'passed') process.exitCode = 1; })
    .catch(error => {
      const allowed = new Set(['noise_stack_required_symbol_missing', 'noise_stack_required_call_missing',
        'noise_stack_required_entry_missing', 'noise_crypto_boundary_missing', 'noise_crypto_entry_missing',
        'noise_stack_config', 'noise_audit_arguments',
        'noise_audit_private_root', 'noise_audit_native_binding', 'noise_audit_objdump_pin']);
      console.error(JSON.stringify({ status: 'blocked', category: allowed.has(error.message) ? error.message : 'noise_stack_audit_failed' }));
      process.exitCode = 1;
    });
}
