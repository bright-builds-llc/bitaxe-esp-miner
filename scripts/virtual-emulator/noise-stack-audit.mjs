import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile, stat, lstat } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { sourceSnapshot } from './identity.mjs';

const PROFILE = 'noise-only-valid-seed1';
const MAIN = 16384;
const MARGIN = 2048;
const OBJDUMP_VERSION = 'esp-14.2.0_20260121';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const hexDigest = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const moduleDirectory = dirname(fileURLToPath(import.meta.url));

function nativeCalls(body) {
  const calls = [], registers = new Map();
  let indirectCalls = 0;
  for (const line of body.split('\n')) {
    const maybeInstruction = line.match(/^\s*[a-f0-9]+:\s+[a-f0-9 ]+\s+([a-z0-9.]+)\s*(.*)$/);
    if (!maybeInstruction) continue;
    const [, op, operands] = maybeInstruction;
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
  const labels = [...disassembly.matchAll(/^([a-f0-9]+) <(.+)>:\s*$/gm)];
  for (let index = 0; index < labels.length; index++) {
    const label = labels[index], address = Number.parseInt(label[1], 16);
    const body = disassembly.slice(label.index + label[0].length, labels[index + 1]?.index ?? disassembly.length);
    const maybeEntry = body.match(/^\s*[a-f0-9]+:\s+[a-f0-9 ]+\s+entry\s+a1,\s*(0x[a-f0-9]+|[0-9]+)/m);
    const observed = nativeCalls(body);
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

const PREFIX = ['bitaxe_virtual_firmware::main', 'bitaxe_virtual_firmware::guest::run',
  'bitaxe_virtual_firmware::noise_probe::run_and_emit', 'bitaxe_simulation::noise_probe::run',
  'bitaxe_simulation::noise_probe::handshake_and_frame'];
const REQUIRED_PATHS = [
  { id: 'completion', names: [...PREFIX, 'bitaxe_stratum::v2::noise::NoiseInitiator::complete_diagnostic',
    'noise_sv2::initiator::Initiator::step_2_with_now', 'noise_sv2::handshake::HandshakeOp::mix_hash'] },
  { id: 'encrypted_frame', names: [...PREFIX, 'bitaxe_simulation::noise_probe::frame_round_trip'] },
];

/** Audit only the selected synthetic valid probe; source branches are not runtime observations. */
export function auditNoiseStack(disassembly, config) {
  const lines = config.split('\n');
  if (lines.filter(line => line.startsWith('CONFIG_ESP_MAIN_TASK_STACK_SIZE=')).length !== 1 ||
      !lines.includes(`CONFIG_ESP_MAIN_TASK_STACK_SIZE=${MAIN}`) ||
      !lines.includes('CONFIG_FREERTOS_CHECK_STACKOVERFLOW_CANARY=y') ||
      !lines.includes('CONFIG_FREERTOS_TASK_FUNCTION_WRAPPER=y') ||
      !lines.includes('# CONFIG_LIBC_NEWLIB_NANO_FORMAT is not set')) throw Error('noise_stack_config');
  const frames = parseNoiseFrames(disassembly);
  const paths = REQUIRED_PATHS.map(({ id, names }) => {
    const path = measureNoisePath(frames, names);
    return { id, ...path, fit: path.frame_bytes <= MAIN - MARGIN };
  });
  return { schema: 'bitaxe-noise-stack-audit-v1', profile: PROFILE, main_stack_bytes: MAIN,
    sdk_extra_stack_bytes: 512, required_margin_bytes: MARGIN, available_selected_path_bytes: MAIN - MARGIN,
    selected_path_budget_fit: paths.every(path => path.fit), paths, complete_callgraph_bound: false,
    scope: 'named_direct_native_paths_only', runtime_high_water_required_bytes: MARGIN,
    initiator_responder_inlined_construction_not_separately_bounded: true,
    crypto_branch_closure_not_bounded: true };
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
  if (receipt?.schema !== 'bitaxe-noise-stack-audit-v1' || receipt.profile !== PROFILE ||
      receipt.selected_path_budget_fit !== true || receipt.complete_callgraph_bound !== false ||
      receipt.main_stack_bytes !== MAIN || receipt.required_margin_bytes !== MARGIN ||
      receipt.available_selected_path_bytes !== MAIN - MARGIN || receipt.sdk_extra_stack_bytes !== 512 ||
      receipt.runtime_high_water_required_bytes !== MARGIN || receipt.scope !== 'named_direct_native_paths_only' ||
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
        !Array.isArray(paths[0].native_symbols) || required.names.some(name => !paths[0].native_symbols.includes(name))) {
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
        'noise_stack_required_entry_missing', 'noise_stack_config', 'noise_audit_arguments',
        'noise_audit_private_root', 'noise_audit_native_binding', 'noise_audit_objdump_pin']);
      console.error(JSON.stringify({ status: 'blocked', category: allowed.has(error.message) ? error.message : 'noise_stack_audit_failed' }));
      process.exitCode = 1;
    });
}
