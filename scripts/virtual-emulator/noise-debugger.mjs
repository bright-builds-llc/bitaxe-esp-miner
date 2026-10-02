import { readFile, writeFile, stat } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { managedGdb } from '../core-dump/main.mjs';
import { runPrivate } from './process.mjs';
import { nativeElfSymbols } from './elf-symbols.mjs';

const directory = dirname(fileURLToPath(import.meta.url));
const ABI = { TCB_BYTES: 340, STACK_OFFSET: 48, END_OFFSET: 72, TOP_OFFSET: 0, NAME_OFFSET: 52,
  PANIC_BYTES: 36, FRAME_OFFSET: 28, EXCEPTION_BYTES: 112, JOURNAL_BYTES: 768, COUNT_BYTES: 4, CURRENT_BYTES: 8,
  REACHED_BYTES: 4, RELEASED_BYTES: 4 };
const digest = value => createHash('sha256').update(value).digest('hex');
const OBJECTS = { JOURNAL_BYTES: ['BITAXE_VIRTUAL_NOISE_CHECKPOINTS', 768],
  COUNT_BYTES: ['BITAXE_VIRTUAL_NOISE_CHECKPOINT_COUNT', 4],
  REACHED_BYTES: ['BITAXE_VIRTUAL_NOISE_PREFIX_REACHED', 4],
  RELEASED_BYTES: ['BITAXE_VIRTUAL_NOISE_PREFIX_RELEASED', 4] };

/** Object sizes come from the exact image when Rust globals have no DWARF types. */
export function noiseObjectAbi(elf) {
  const symbols = nativeElfSymbols(elf);
  return Object.entries(OBJECTS).map(([key, [name, bytes]]) => {
    const matches = symbols.filter(symbol => symbol.name === name);
    if (matches.length !== 1 || matches[0].type !== 1 || !matches[0].defined || matches[0].bytes !== bytes) throw Error('noise_debugger_object_abi');
    return `ABI_${key} ${bytes}`;
  }).join('\n');
}

/** SDK types come from the actual ELF; no live inferior function is called. */
export function parseNoiseDebuggerAbi(stdout) {
  const values = {};
  for (const [name, required] of Object.entries(ABI)) {
    const matches = [...stdout.matchAll(new RegExp(`^ABI_${name} ([0-9]+)$`, 'gm'))];
    if (matches.length !== 1 || Number(matches[0][1]) !== required) throw Error('noise_debugger_abi');
    values[name.toLowerCase()] = required;
  }
  return values;
}

/** Compile a reviewed GDB file; only the wrapper-owned loopback endpoint is accepted. */
export async function noiseDebuggerScript({ root, port, selectedStop }) {
  if (!/^\/[a-zA-Z0-9_./-]+$/.test(root) || !Number.isInteger(port) || port < 1024 || port > 65535 ||
      ![101, 102, 103, 105, 106, 107, 109, 110].includes(selectedStop)) throw Error('noise_debugger_arguments');
  const capture = await readFile(join(directory, 'noise-debugger-capture.gdb'), 'utf8');
  const template = await readFile(join(directory, 'noise-debugger.gdb'), 'utf8');
  return template.replaceAll('@CUTOFF@', String(selectedStop)).replaceAll('@PORT@', String(port)).replaceAll('@CAPTURE_0@', capture.replaceAll('@INDEX@', '0').replaceAll('@ROOT@', root))
    .replaceAll('@CAPTURE_1@', capture.replaceAll('@INDEX@', '1').replaceAll('@ROOT@', root));
}

export function noiseDebuggerArguments(elf, script) {
  return ['--nx', '--quiet', '--batch', '-iex', 'set auto-load no', '-iex', 'set debuginfod enabled off', elf, '-x', script];
}

/** Offline preflight only. Runtime wrappers own GDB/QEMU bounds and listener release. */
export async function prepareNoiseDebugger(repo, root, elf, port, selectedStop) {
  if (((await stat(root)).mode & 0o777) !== 0o700) throw Error('noise_debugger_private_root');
  const gdb = await managedGdb(repo);
  const preflight = join(directory, 'noise-debugger-preflight.gdb');
  let stdout;
  try {
    stdout = execFileSync(gdb.path, noiseDebuggerArguments(elf, preflight), { encoding: 'utf8', timeout: 25000,
      maxBuffer: 2 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (error) {
    await writeFile(join(root, 'gdb-preflight.stdout.private'), error.stdout ?? '', { flag: 'wx', mode: 0o600 });
    await writeFile(join(root, 'gdb-preflight.stderr.private'), error.stderr ?? '', { flag: 'wx', mode: 0o600 });
    throw Error('noise_debugger_preflight');
  }
  await writeFile(join(root, 'gdb-preflight.stdout.private'), stdout, { flag: 'wx', mode: 0o600 });
  const elfBytes = await readFile(elf);
  const abi = parseNoiseDebuggerAbi(`${stdout}\n${noiseObjectAbi(elfBytes)}\n`);
  const script = join(root, 'noise-capture.gdb');
  const content = await noiseDebuggerScript({ root, port, selectedStop });
  await writeFile(script, content, { flag: 'wx', mode: 0o600 });
  // Parse definitions and breakpoint commands offline; the remote attach and resume are omitted.
  const grammar = join(root, 'noise-capture-offline-grammar.gdb');
  await writeFile(grammar, content.replace(/^target remote .+\n/m, '').replace(/^continue\n?$/m, ''), { flag: 'wx', mode: 0o600 });
  try {
    const checked = execFileSync(gdb.path, noiseDebuggerArguments(elf, grammar), { encoding: 'utf8', timeout: 25000,
      maxBuffer: 2 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
    await writeFile(join(root, 'gdb-grammar.stdout.private'), checked, { flag: 'wx', mode: 0o600 });
  } catch (error) {
    await writeFile(join(root, 'gdb-grammar.stdout.private'), error.stdout ?? '', { flag: 'wx', mode: 0o600 });
    await writeFile(join(root, 'gdb-grammar.stderr.private'), error.stderr ?? '', { flag: 'wx', mode: 0o600 });
    throw Error('noise_debugger_offline_grammar');
  }
  const identity = { schema: 'bitaxe-noise-debugger-preflight-v1', elf_sha256: digest(elfBytes),
    gdb_version: gdb.version, gdb_sha256: gdb.sha256, abi, live_effects: false,
    rust_object_size_source: 'exact_elf_symbol_table', sdk_type_source: 'exact_elf_dwarf',
    output_limit_bytes: 2 * 1024 * 1024, debugger_timeout_ms: 25000, snapshots_maximum: 2 };
  await writeFile(join(root, 'gdb-preflight.json'), JSON.stringify(identity), { flag: 'wx', mode: 0o600 });
  return { command: gdb.path, args: noiseDebuggerArguments(elf, script), identity };
}

/** Decode after both owners stop; raw debugger output stays in the protected root. */
export async function decodeNoiseDebugger(repo, root, selectedStop, stdoutPath = join(root, 'noise-gdb.stdout.log')) {
  const output = join(root, 'noise-debugger-decoded.json');
  await runPrivate('python3', [join(repo, 'scripts/virtual-emulator/noise-debugger-decode.py'), root, stdoutPath,
    String(selectedStop), output], root, 'noise-debugger-decode', { cwd: repo, timeoutMs: 25000, maxOutputBytes: 2 * 1024 * 1024 });
  return JSON.parse(await readFile(output, 'utf8'));
}

/** Material closure belongs in the wrapper identity before any live capture. */
export const noiseDebuggerMaterials = ['noise-debugger.mjs', 'noise-debugger.gdb', 'noise-debugger-capture.gdb',
  'noise-debugger-preflight.gdb', 'noise-debugger-decode.py', 'elf-symbols.mjs', 'process.mjs', '../core-dump/main.mjs'];
