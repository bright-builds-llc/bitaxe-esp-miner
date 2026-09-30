import { parseCheckpoints } from './checkpoints.mjs';
import { createServer } from 'node:net';
import { createHash } from 'node:crypto';
import { readFile, writeFile, chmod } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';
import { managedGdb } from '../core-dump/main.mjs';
import { doctor, managedPaths } from './setup.mjs';
import { MARKER, PROFILE } from './build.mjs';
import { qemuArguments, writeSdkEfuse } from './run.mjs';
import { runPrivate } from './process.mjs';
const sha = value => createHash('sha256').update(value).digest('hex');

/** One published healthy diagnostic; private live GDB stops at first failure or fourth checkpoint. */
export async function diagnoseHeap(repo, packagePath, root) {
  const manifestBytes = await readFile(packagePath);
  const manifest = JSON.parse(manifestBytes);
  if (manifest.execution_profile !== PROFILE || manifest.hardware_eligible !== false) throw Error('guest_package_profile');
  if (![manifest.virtual_elf, manifest.flash_image].every(name => typeof name === 'string' && /^[a-zA-Z0-9_.-]+$/.test(name))) throw Error('guest_artifact_path');
  const elf = await readFile(join(dirname(packagePath), manifest.virtual_elf));
  const image = await readFile(join(dirname(packagePath), manifest.flash_image));
  if (sha(elf) !== manifest.virtual_elf_sha256 || !elf.includes(Buffer.from(MARKER)) || sha(image) !== manifest.image_sha256) throw Error('guest_package_digest');
  const elfPath = join(root, 'virtual-ultra205.elf'), flash = join(root, 'virtual-flash.bin');
  await writeFile(elfPath, elf, { flag: 'wx', mode: 0o600 }); await writeFile(flash, image, { flag: 'wx', mode: 0o600 });
  await doctor(repo, root);
  const tools = managedPaths(repo), debuggerTool = await managedGdb(repo);
  const preflightScript = join(root, 'debugger-preflight.gdb');
  await writeFile(preflightScript, ['set auto-load no', 'set pagination off', 'set print frame-arguments none',
    'set debuginfod enabled off', 'set tcp auto-retry on', 'set tcp connect-timeout 5', 'set language c',
    'info address BITAXE_VIRTUAL_HEAP_CHECKPOINTS', 'p sizeof(unsigned char[384])',
    'info scope bitaxe_virtual_checkpoint_complete'].join('\n') + '\n', { flag: 'wx', mode: 0o600 });
  try { await runPrivate(debuggerTool.path, ['--nx', '--batch', elfPath, '-x', preflightScript], root, 'gdb-preflight', { timeoutMs: 10000 }); }
  catch { throw Error('emulator_debugger_preflight_failed'); }
  const preflightOutput = await readFile(join(root, 'gdb-preflight.stdout.log'), 'utf8');
  if (!/\$[0-9]+ = 384\b/.test(preflightOutput) || !preflightOutput.includes('variable in $a2, length 4')) throw Error('emulator_debugger_preflight_symbols');
  const sdkTools = JSON.parse(await readFile(join(tools.idf, 'tools/tools.json'), 'utf8'));
  const compiler = sdkTools.tools.find(tool => tool.name === 'xtensa-esp-elf')?.versions.find(version => version.status === 'recommended');
  if (!compiler) throw Error('guest_sdk_compiler_missing');
  await runPrivate(join(repo, '.embuild/espressif/tools/xtensa-esp-elf', compiler.name, 'xtensa-esp-elf/bin/xtensa-esp32s3-elf-objdump'),
    ['-t', elfPath], root, 'checkpoint-abi', { timeoutMs: 10000 });
  if (!/\b00000180\s+BITAXE_VIRTUAL_HEAP_CHECKPOINTS$/m.test(await readFile(join(root, 'checkpoint-abi.stdout.log'), 'utf8'))) throw Error('checkpoint_symbol_size');
  const server = createServer();
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const port = server.address().port; await new Promise(resolve => server.close(resolve));
  const script = join(root, 'diagnose.gdb');
  await writeFile(script, [
    'set pagination off', 'set print frame-arguments none', 'set auto-load no', 'set debuginfod enabled off',
    'set tcp auto-retry on', 'set tcp connect-timeout 5', `target remote 127.0.0.1:${port}`,
    'break __assert_func', 'break esp_panic_handler', 'break bitaxe_virtual_checkpoint_complete if $a2 == 4', 'continue',
    'thread apply all bt 80', 'info args', 'info registers', 'set language c',
    `dump binary memory ${join(root, 'checkpoints.raw')} (char*)&BITAXE_VIRTUAL_HEAP_CHECKPOINTS ((char*)&BITAXE_VIRTUAL_HEAP_CHECKPOINTS+384)`,
    'detach',
  ].join('\n') + '\n', { flag: 'wx', mode: 0o600 });
  const efuse = await writeSdkEfuse(repo, root);
  const qemu = runPrivate(tools.binary, [...qemuArguments(flash, efuse),
    '-gdb', `tcp:127.0.0.1:${port}`, '-S'], root, 'qemu', { timeoutMs: 30000, allowTimeout: true, inputReadyMarker: '"event":"task"',
    input: `${JSON.stringify({ command: 'scenario', scenario: 'healthy-lifecycle', seed: 1, diagnose_heap: true })}\n` }).then(value => ({ value }), error => ({ error: error.message }));
  await wait(200);
  let maybeDebuggerFailure = null;
  try { await runPrivate(debuggerTool.path, ['--nx', '--batch', elfPath, '-x', script], root, 'gdb', { timeoutMs: 25000 }); }
  catch (error) { maybeDebuggerFailure = error.message; }
  const qemuOutcome = await qemu;
  if (qemuOutcome.error && !maybeDebuggerFailure) maybeDebuggerFailure = 'diagnostic_qemu_failed';
  let records = [];
  try {
    const raw = await readFile(join(root, 'checkpoints.raw')); await chmod(join(root, 'checkpoints.raw'), 0o600);
    records = parseCheckpoints(raw);
  } catch (error) { if (!maybeDebuggerFailure) maybeDebuggerFailure = 'checkpoint_capture_unavailable'; }
  const result = { schema: 'bitaxe-virtual-heap-diagnostic-v2', scenario: 'healthy-lifecycle', seed: 1,
    compiled_source_sha256: manifest.compiled_source_sha256, elf_sha256: manifest.virtual_elf_sha256,
    records, maybe_debugger_failure: maybeDebuggerFailure, process_released: qemuOutcome.value?.released === true,
    physical_cause_proven: false, qualification: false };
  await writeFile(join(root, 'diagnostic-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  return result;
}
