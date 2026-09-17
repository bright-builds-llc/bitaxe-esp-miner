import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { readFile, realpath, stat } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { promisify } from 'node:util';
import { auditOwnerStack } from './owner-stack-audit.mjs';
import { auditTelemetryStack, parseNativeFunctions, nativeFrame as staticFrame } from './telemetry-stack-audit.mjs';
import { noiseNativeCalls as nativeCalls, noiseNativeInstructions } from './noise-native-calls.mjs';
import { resolveNoiseInstructions, selectedNoiseSymbol } from './noise-native-disassembly.mjs';
const nativeFrame = fn => staticFrame({ ...fn, instructions: noiseNativeInstructions(fn) });

const run = promisify(execFile);
const sha = value => createHash('sha256').update(value).digest('hex');
const check = (condition, reason) => { if (!condition) throw Object.assign(Error(reason), { code: reason }); };
export const NOISE_STACK_BYTES = 12288;
export const NOISE_ENTRY_BUDGET = 8192;
export const NOISE_PLATFORM_MARGIN_BYTES = 512;
export const NOISE_ENTRY_SYMBOL = 'bitaxe_noise_serial_owner_entry';
export const NATIVE_AUDITOR_SOURCES = Object.freeze(['scripts/noise-native-readiness.mjs', 'scripts/noise-native-calls.mjs',
  'scripts/noise-native-disassembly.mjs', 'scripts/owner-stack-audit.mjs', 'scripts/telemetry-stack-audit.mjs']);
const CONFIGURATION = [
  'CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304',
  'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384', 'CONFIG_ESP_MAIN_TASK_AFFINITY=0x0',
  'CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5', 'CONFIG_SPIRAM_TRY_ALLOCATE_WIFI_LWIP=y',
  'CONFIG_ESP_WIFI_STATIC_RX_BUFFER_NUM=6', 'CONFIG_ESP_WIFI_STATIC_TX_BUFFER_NUM=6',
  'CONFIG_ESP_WIFI_TX_BUFFER_TYPE=0', 'CONFIG_ESP_WIFI_DYNAMIC_RX_BUFFER_NUM=32',
  'CONFIG_ESP_WIFI_AMPDU_RX_ENABLED=y', 'CONFIG_ESP_WIFI_RX_BA_WIN=12',
];

/** Selected native entry frame only; never a whole-worker or hardware-fit assertion. */
export function auditNoiseEntry(disassembly, ownerSource, transportSource) {
  check(!/thread::Builder|\.spawn\(/u.test(ownerSource)
    && /const WORKER_STACK_BYTES:\s*usize\s*=\s*12\s*\*\s*1024\s*;/u.test(transportSource)
    && /\.stack_size\(WORKER_STACK_BYTES\)/u.test(transportSource), 'noise_native_stack_declaration');
  const sections = disassembly.split(/(?=^[0-9a-f]+ <[^\n]+>:\s*$)/mu);
  const matching = sections.filter(section => new RegExp(`^[0-9a-f]+ <${NOISE_ENTRY_SYMBOL}>:\\s*$`, 'mu').test(section.split('\n')[0]));
  check(matching.length === 1, 'noise_native_entry_missing_or_multiple');
  const instructions = matching[0].split('\n').slice(1).filter(line => /^\s*[0-9a-f]+:\s+[0-9a-f]+\s+/u.test(line));
  const entries = [...matching[0].matchAll(/\bentry\s+a1,\s*(0x[0-9a-f]+|[0-9]+)\s*$/gmu)];
  check(entries.length === 1 && /\bentry\s+a1,/u.test(instructions[0] ?? ''), 'noise_native_entry_frame');
  const bytes = Number(entries[0][1]);
  check(Number.isSafeInteger(bytes) && bytes >= 32 && bytes % 16 === 0 && bytes <= NOISE_ENTRY_BUDGET, 'noise_native_entry_budget');
  check(!/\b(?:add|addi|addmi|sub|mov|movsp)(?:\.n)?\s+a1\s*,/u.test(matching[0]), 'noise_native_dynamic_stack');
  return { symbol: NOISE_ENTRY_SYMBOL, stackBytes: NOISE_STACK_BYTES, entryBytes: bytes,
    entryBudgetBytes: NOISE_ENTRY_BUDGET, completeCallgraphBound: false };
}

/** Explicit resolved crypto paths plus the borrowed worker callers; unknown paths are not certified. */
export function auditBorrowedNoisePaths(disassembly, ownerSource, transportSource, borrowSource, resolvedFunctions, { compilerPrivateSpills = false } = {}) {
  const calls = (fn) => nativeCalls(fn, { compilerPrivateSpills });
  const entry = auditNoiseEntry(disassembly, ownerSource, transportSource);
  const registered = ownerSource.includes('.dispatch(owner_entry, job_completed)') ||
    /\.dispatch\(\s*if owner\.external \{\s*crate::v2_serial_runtime::channel_entry\s*\} else \{\s*owner_entry\s*\},\s*job_completed,\s*\)/u.test(ownerSource);
  check(registered && transportSource.includes('borrow.run_job()')
    && borrowSource.includes('(callbacks.run)();') && borrowSource.includes('(callbacks.complete)();'), 'noise_native_callback_binding');
  const functions = resolvedFunctions ?? parseNativeFunctions(disassembly), all = [...functions.values()];
  const one = symbol => { const found = all.filter(fn => fn.symbol === symbol); check(found.length === 1, 'noise_native_path_symbol'); return found[0]; };
  const loop = one('bitaxe_firmware::production_mining_session::transport::run_worker');
  const dispatch = one('bitaxe_firmware::production_mining_session::transport::borrow::NoiseBorrowWorker::run_job');
  check(calls(loop).some(edge => edge.target === dispatch.address), 'noise_native_borrow_edge');
  const caller = (callee, symbol) => {
    const found = all.filter(fn => fn.symbol === symbol && fn.instructions.some(instruction => instruction.args.includes(`${callee.address.toString(16)} <`))
      && calls(fn).some(edge => edge.target === callee.address));
    check(found.length === 1, 'noise_native_thread_caller'); return found[0];
  };
  const backtrace = caller(loop, 'std::sys::backtrace::__rust_begin_short_backtrace');
  const shim = caller(backtrace, 'core::ops::function::FnOnce::call_once{{vtable.shim}}');
  const entries = [one('pthread_task_func'), one('std::sys::pal::unix::thread::Thread::new::thread_start'), shim, backtrace];
  const start = one(NOISE_ENTRY_SYMBOL), observed = new Set(), memo = new Map(), active = new Set();
  const total = path => path.reduce((sum, node) => sum + node.entryBytes, 0);
  const visit = fn => {
    check(!active.has(fn.address), 'noise_native_selected_recursion');
    if (memo.has(fn.address)) return memo.get(fn.address);
    check(observed.size < 4096, 'noise_native_selected_bound');
    active.add(fn.address); observed.add(fn.symbol);
    let longest = [];
    for (const edge of calls(fn)) {
      const target = functions.get(edge.target);
      if (target && selectedNoiseSymbol(target.symbol)) { const path = visit(target); if (total(path) > total(longest)) longest = path; }
    }
    const result = [{ symbol: fn.symbol, entryBytes: nativeFrame(fn) }, ...longest];
    active.delete(fn.address); memo.set(fn.address, result); return result;
  };
  const suffix = visit(start);
  check([...observed].some(name => /schnorrsig_verify/u.test(name)) && [...observed].some(name => /ellswift_encode/u.test(name)), 'noise_native_crypto_path_unproven');
  const worst = [...entries, loop, dispatch].map(fn => ({ symbol: fn.symbol, entryBytes: nativeFrame(fn) })).concat(suffix);
  const bytes = total(worst);
  if (bytes + NOISE_PLATFORM_MARGIN_BYTES > NOISE_STACK_BYTES) throw Object.assign(Error('noise_native_selected_path_budget'), {
    code: 'noise_native_selected_path_budget', selectedPathBytes: bytes, nodes: worst, requiredMarginBytes: NOISE_PLATFORM_MARGIN_BYTES,
  });
  return { ...entry, ownership: 'existing_primary_transport_worker', addedStackBytes: 0,
    selectedPathBytes: bytes, selectedFunctions: memo.size, nodes: worst,
    requiredMarginBytes: NOISE_PLATFORM_MARGIN_BYTES, remainingStackBytes: NOISE_STACK_BYTES - bytes,
    callbackBinding: 'source_bound_fn_pointer_registration', threadEntryBinding: 'pthread_rust_vtable_runtime', completeCallgraphBound: false };
}

export function requireNoiseConfiguration(sdkconfig) {
  const lines = sdkconfig.split(/\r?\n/u);
  for (const expected of CONFIGURATION) {
    const key = expected.slice(0, expected.indexOf('=') + 1);
    const rows = lines.filter(line => line.startsWith(key));
    check(rows.length === 1 && rows[0] === expected, 'noise_native_configuration');
  }
}

function artifact(manifest, kind, expectedPath) {
  const rows = manifest.artifacts.filter(item => item.kind === kind);
  check(rows.length === 1 && rows[0].path === expectedPath
    && /^[a-f0-9]{64}$/u.test(rows[0].sha256), 'noise_native_artifact');
  return rows[0];
}

/** Read-only inspection of the exact canonical native package, before attempt assignment. */
export async function inspectNoiseNativeReadiness({ firmwareRoot, manifestPath, expectedSourceCommit, expectedElfSha256 }, { compilerPrivateSpills = false } = {}) {
  check(/^[a-f0-9]{40}$/u.test(expectedSourceCommit) && /^[a-f0-9]{64}$/u.test(expectedElfSha256), 'noise_native_identity');
  const root = await realpath(firmwareRoot);
  const canonical = await realpath(join(root, 'bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json'));
  check(await realpath(manifestPath) === canonical, 'noise_native_canonical_manifest');
  const manifestBytes = await readFile(canonical), manifest = JSON.parse(manifestBytes);
  check(manifest.schema_version === 4 && manifest.source_commit === expectedSourceCommit
    && manifest.app_elf_sha256 === expectedElfSha256 && manifest.build_identity?.source_dirty === false
    && manifest.image_metadata?.board === '205' && manifest.image_metadata.esp_idf_version === 'v5.5.4'
    && Array.isArray(manifest.artifacts), 'noise_native_package_identity');
  const elfArtifact = artifact(manifest, 'firmware_elf', 'bitaxe-ultra205.elf'), appArtifact = artifact(manifest, 'firmware_ota_image', 'esp-miner.bin');
  const directory = dirname(canonical), elfPath = join(directory, elfArtifact.path), appPath = join(directory, appArtifact.path);
  check(dirname(await realpath(elfPath)) === directory && dirname(await realpath(appPath)) === directory, 'noise_native_artifact_alias');
  const elf = await readFile(elfPath), app = await readFile(appPath);
  check(sha(elf) === expectedElfSha256 && sha(elf) === elfArtifact.sha256 && sha(app) === appArtifact.sha256, 'noise_native_artifact_changed');
  check(app.length > 0 && app.length <= 4 * 1024 * 1024, 'noise_native_image_slot');
  const configPath = join(directory, 'bitaxe-firmware.sdkconfig'), sdkconfig = await readFile(configPath, 'utf8');
  requireNoiseConfiguration(sdkconfig);
  const noiseSourcePath = join(root, 'firmware/bitaxe/src/noise_serial_runtime.rs');
  const productionSourcePath = join(root, 'firmware/bitaxe/src/production_mining_session.rs');
  const transportSourcePath = join(root, 'firmware/bitaxe/src/production_mining_session/transport.rs');
  const borrowSourcePath = join(root, 'firmware/bitaxe/src/production_mining_session/transport/borrow.rs');
  const noiseSource = await readFile(noiseSourcePath, 'utf8'), productionSource = await readFile(productionSourcePath, 'utf8');
  const transportSource = await readFile(transportSourcePath, 'utf8'), borrowSource = await readFile(borrowSourcePath, 'utf8');
  const tool = join(root, '.embuild/espressif/tools/xtensa-esp-elf/esp-14.2.0_20260121/xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump');
  check((await stat(tool)).isFile(), 'noise_native_objdump');
  const toolSha256 = sha(await readFile(tool));
  const { stdout: disassembly } = await run(tool, ['-Cd', elfPath], {
    timeout: 30000, maxBuffer: 128 * 1024 * 1024, env: { PATH: '/usr/bin:/bin', LANG: 'C', LC_ALL: 'C' },
  });
  const decodeDeadline = performance.now() + 30000;
  const decoded = await resolveNoiseInstructions(parseNativeFunctions(disassembly), disassembly, async (start, end) => {
    check(performance.now() < decodeDeadline, 'noise_native_decode_timeout');
    const value = await run(tool, ['-Cd', `--start-address=0x${start.toString(16)}`, `--stop-address=0x${end.toString(16)}`, elfPath], {
      timeout: Math.max(1, Math.floor(decodeDeadline - performance.now())), maxBuffer: 1024 * 1024,
      env: { PATH: '/usr/bin:/bin', LANG: 'C', LC_ALL: 'C' },
    }); return value.stdout;
  }, { compilerPrivateSpills });
  const noise = { ...auditBorrowedNoisePaths(disassembly, noiseSource, transportSource, borrowSource, decoded.functions, { compilerPrivateSpills }),
    supplementalRanges: decoded.supplementalRanges, unresolvedJumps: decoded.unresolvedJumps };
  const production = auditOwnerStack(disassembly, productionSource);
  const telemetry = auditTelemetryStack(disassembly, sdkconfig);
  check(telemetry.result === 'targeted_path_fits', 'noise_native_telemetry_budget');
  const sources = [];
  for (const path of NATIVE_AUDITOR_SOURCES) sources.push({ path, sha256: sha(await readFile(join(root, path))) });
  // Recheck every measured mutable input before returning a source-bound receipt.
  for (const [path, bytes] of [[canonical, manifestBytes], [elfPath, elf], [appPath, app], [configPath, sdkconfig],
    [noiseSourcePath, noiseSource], [productionSourcePath, productionSource], [transportSourcePath, transportSource], [borrowSourcePath, borrowSource]]) {
    check(sha(await readFile(path)) === sha(bytes), 'noise_native_input_changed');
  }
  check(sha(await readFile(tool)) === toolSha256, 'noise_native_tool_changed');
  return { schema: 'noise-serial-native-readiness-v1', result: 'selected_native_checks_passed',
    firmwareCommit: expectedSourceCommit, elfSha256: expectedElfSha256, packageManifestSha256: sha(manifestBytes),
    appImageSha256: sha(app), appImageBytes: app.length, imageSlotBytes: 4 * 1024 * 1024,
    sdkconfigSha256: sha(sdkconfig), noiseOwnerSourceSha256: sha(noiseSource), productionOwnerSourceSha256: sha(productionSource),
    transportOwnerSourceSha256: sha(transportSource), borrowSourceSha256: sha(borrowSource),
    objdumpSha256: toolSha256, auditorSources: sources, noise, production, telemetry,
    hardwareQualified: false, startupHeapQualified: false, completeCallgraphBound: false };
}
