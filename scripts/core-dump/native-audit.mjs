import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { check, sha256, privateDirectory } from './files.mjs';

const iram = address => address >= 0x40370000 && address < 0x403e0000;
const dram = address => address >= 0x3fc80000 && address < 0x3fd00000;
function symbols(text) {
  return text.split('\n').flatMap(line => {
    const match = /^([a-f0-9]+)\s+([a-f0-9]+)\s+[A-Za-z]\s+(.+)$/u.exec(line);
    return match ? [{ address: parseInt(match[1], 16), size: parseInt(match[2], 16), name: match[3] }] : [];
  });
}
function instructions(text) {
  return text.split('\n').flatMap(line => {
    const match = /^\s*([a-f0-9]+):\s+[a-f0-9]+\s+(\S+)\s*(.*)$/u.exec(line);
    return match ? [{ address: parseInt(match[1], 16), op: match[2], args: match[3] }] : [];
  });
}
function wordAt(elf, address) {
  check(elf.subarray(0, 7).equals(Buffer.from([127, 69, 76, 70, 1, 1, 1])) && elf.readUInt16LE(18) === 94, 'native_elf_arch');
  const offset = elf.readUInt32LE(28), size = elf.readUInt16LE(42), count = elf.readUInt16LE(44);
  check(size === 32 && count < 256, 'native_elf_headers');
  for (let i = 0; i < count; i++) {
    const entry = offset + i * size; check(entry + size <= elf.length, 'native_elf_headers');
    const start = elf.readUInt32LE(entry + 8), length = elf.readUInt32LE(entry + 16);
    if (elf.readUInt32LE(entry) !== 1 || address < start || address + 4 > start + length) continue;
    const file = elf.readUInt32LE(entry + 4) + address - start; check(file + 4 <= elf.length, 'native_literal_bound');
    return elf.readUInt32LE(file);
  }
  throw Error('native_literal_unmapped');
}

/** Fail closed on the exact native cutoff instruction and placement contract. */
export function audit(elf, symbolText, wrapperText, portText) {
  const entries = symbols(symbolText);
  const one = predicate => { const found = entries.filter(predicate); check(found.length === 1, 'native_symbol'); return found[0]; };
  const wrapper = one(row => row.name === '__wrap_esp_panic_handler');
  const real = one(row => row.name === 'esp_panic_handler');
  const receipt = one(row => row.name === 'BITAXE_PANIC_CUTOFF_RECEIPT');
  const gate = one(row => row.name.includes('revocation6global4GATE'));
  const configured = one(row => row.name.includes('panic_cutoff18CONFIGURED_OUTPUTS'));
  const selfTest = one(row => row.name.includes('panic_cutoff16SELF_TEST_MARKER'));
  check(iram(wrapper.address) && iram(wrapper.address + wrapper.size - 1) && wrapper.size <= 512, 'native_wrapper_iram');
  for (const item of [receipt, gate, configured, selfTest])
    check(dram(item.address) && dram(item.address + item.size - 1), 'native_state_dram');
  check(receipt.size === 28, 'native_receipt_size');
  const code = instructions(wrapperText).filter(row => row.address >= wrapper.address && row.address < wrapper.address + wrapper.size);
  check(code.length > 0 && code.length <= 96 && code[0].op === 'entry' && /^a1, (?:16|32|48|64)$/u.test(code[0].args), 'native_frame');
  const regs = new Map(), stores = [], calls = [];
  const allowed = new Set(['entry', 'l32r', 'movi', 'movi.n', 'memw', 's32i', 's32i.n', 'l32i', 'l32i.n', 'and', 'or', 'srli', 'mov.n', 'callx8', 'retw.n']);
  for (const row of code) {
    check(allowed.has(row.op), 'native_cutoff_instruction');
    const args = row.args.split(',').map(part => part.trim());
    if (row.op === 'l32r') {
      const address = parseInt(args[1], 16); check(iram(address), 'native_literal_iram');
      regs.set(args[0], wordAt(elf, address));
    } else if (row.op.startsWith('movi')) regs.set(args[0], Number(args[1]) >>> 0);
    else if (row.op === 'mov.n') regs.set(args[0], regs.get(args[1]));
    else if (row.op.startsWith('l32i')) regs.set(args[0], { load: (regs.get(args[1]) + Number(args[2])) >>> 0 });
    else if (row.op === 'and' || row.op === 'or') {
      const left = regs.get(args[1]), right = regs.get(args[2]);
      if (typeof left === 'number' && typeof right === 'number') regs.set(args[0], (row.op === 'and' ? left & right : left | right) >>> 0);
      else regs.set(args[0], { op: row.op, left, right });
    } else if (row.op === 'srli') regs.delete(args[0]);
    else if (row.op.startsWith('s32i')) stores.push({ address: (regs.get(args[1]) + Number(args[2])) >>> 0, value: regs.get(args[0]), at: row.address });
    else if (row.op === 'callx8') calls.push({ address: regs.get(args[0]), at: row.address });
  }
  check(stores[0]?.address === 0x60004008 && stores[0].value === 0x400 &&
    stores[1]?.address === 0x6000400c && stores[1].value === 2, 'native_safe_latch_order');
  check(calls.length === 1 && calls[0].address === real.address && calls[0].at > stores[1].at, 'native_delegate');
  const revocation = stores.find(row => row.address === gate.address);
  check(revocation?.value?.op === 'or' && revocation.value.right === 4 && revocation.value.left?.op === 'and' &&
    revocation.value.left.right === 0xfffffff8 && revocation.value.left.left?.load === gate.address && revocation.at < calls[0].at,
  'native_generation_revoke');
  check(stores.at(-1)?.address === receipt.address && stores.at(-1).value === 0x50434f32 && stores.at(-1).at < calls[0].at, 'native_receipt_commit');
  const port = instructions(portText); let routes = 0;
  for (let i = 0; i + 1 < port.length; i++) {
    if (port[i].op !== 'l32r') continue;
    const [reg, literal] = port[i].args.split(',').map(part => part.trim());
    const target = wordAt(elf, parseInt(literal, 16));
    check(target !== real.address, 'native_port_bypass');
    if (target === wrapper.address && port[i + 1].op === 'callx8' && port[i + 1].args === reg) routes++;
  }
  check(!port.some(row => /^call[048]|^call12|^j$/u.test(row.op) && row.args.includes('<esp_panic_handler>')), 'native_port_bypass');
  check(routes === 1, 'native_port_routing');
  return { schema: 'str005-native-panic-cutoff-audit-v1', elf_sha256: sha256(elf), wrapper_iram: true, literals_iram: true,
    state_internal_dram: true, safe_latches_before_delegate: true, generation_revoked_before_delegate: true,
    no_calls_or_branches_before_cutoff: true, port_routes_wrapper: true, wrapper_instructions: code.length,
    hardware_verified: false };
}

export async function main(argv) {
  check(argv.length === 4 && argv[0] === '--elf' && argv[2] === '--output', 'native_audit_arguments');
  const repo = process.env.BUILD_WORKSPACE_DIRECTORY ?? execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  const manifest = JSON.parse(await readFile(join(repo, '.embuild/espressif/esp-idf/v5.5.4/tools/tools.json'), 'utf8'));
  const tool = manifest.tools.find(item => item.name === 'xtensa-esp-elf');
  const version = tool.versions.find(item => item.status === 'recommended').name;
  const bin = join(repo, '.embuild/espressif/tools/xtensa-esp-elf', version, 'xtensa-esp-elf/bin');
  const elfPath = resolve(argv[1]), output = resolve(argv[3]);
  const run = (name, args) => execFileSync(join(bin, `xtensa-esp32s3-elf-${name}`), args, { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024, timeout: 30000 });
  const result = audit(await readFile(elfPath), run('nm', ['-S', elfPath]),
    run('objdump', ['-d', '--disassemble=__wrap_esp_panic_handler', elfPath]), run('objdump', ['-d', '--disassemble=panic_handler', elfPath]));
  await privateDirectory(resolve(output, '..'));
  await writeFile(output, JSON.stringify(result, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(result => console.log(JSON.stringify(result))).catch(() => {
    console.error('{"native_audit":"blocked"}'); process.exitCode = 1;
  });
}
