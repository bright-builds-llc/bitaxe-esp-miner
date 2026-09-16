import { noiseNativeCalls } from './noise-native-calls.mjs';
const check = (condition, code) => { if (!condition) throw Error(code); };
const noReturn = symbol => /^(?:alloc::alloc::handle_alloc_error|alloc::raw_vec::handle_error|core::panicking::panic\w*|core::result::unwrap_failed|abort)$/u.test(symbol);
const branch = instruction => {
  const match = /(?:^|,\s*)([0-9a-f]+)\s+</u.exec(instruction.args);
  return match ? parseInt(match[1], 16) : null;
};
export const selectedNoiseSymbol = symbol => symbol === 'bitaxe_noise_serial_owner_entry'
  || symbol.startsWith('bitaxe_firmware::noise_serial_runtime::transport::')
  || symbol.startsWith('bitaxe_stratum::v2::noise::') || symbol.includes('noise_sv2::')
  || symbol.startsWith('secp256k1::') || symbol.startsWith('rustsecp256k1_v0_9_2_');

function decode(text) {
  const rows = new Map();
  for (const line of text.split('\n')) {
    const match = /^\s*([0-9a-f]+):\s+([0-9a-f]+)\s+([a-z][a-z0-9_.]*)\s*(.*)$/u.exec(line);
    if (match) rows.set(parseInt(match[1], 16), { address: parseInt(match[1], 16), bytes: match[2].length / 2, op: match[3], args: match[4] });
  }
  return rows;
}

function terminalCall(instruction, reachable) {
  if (!/^callx8$/u.test(instruction.op)) return false;
  let cursor = instruction.address;
  const previous = [...reachable.values()].filter(row => row.address < cursor).sort((a, b) => b.address - a.address);
  for (const row of previous.slice(0, 8)) {
    if (row.address + row.bytes !== cursor) return false;
    if (row.args.startsWith(`${instruction.args},`)) return row.op === 'l32r'
      && /<(?:alloc::alloc::handle_alloc_error|alloc::raw_vec::handle_error|core::panicking::panic\w*|core::result::unwrap_failed|abort)>\)\s*$/u.test(row.args);
    if (/^(?:call|j|b|ret)/u.test(row.op)) return false;
    cursor = row.address;
  }
  return false;
}

/** Decode actual branch starts: linear objdump may consume alignment padding with the next opcode.
 * Only selected functions are repaired; every reachable address must decode within its symbol.
 * The decoder is the exact bound native tool, never a handwritten instruction decoder.
 */
export async function resolveNoiseInstructions(functions, disassembly, readRange) {
  const ordered = [...functions.values()].sort((a, b) => a.address - b.address);
  const rows = decode(disassembly), result = new Map(functions);
  const candidates = ordered.filter(fn => fn.symbol === 'bitaxe_noise_serial_owner_entry');
  check(candidates.length === 1, 'noise_native_decode_root');
  const completed = new Set();
  let ranges = 0, visited = 0;
  const unresolvedJumps = [];
  while (candidates.length) {
    const fn = candidates.pop();
    if (completed.has(fn.address)) continue;
    completed.add(fn.address);
    const index = ordered.indexOf(fn);
    const end = ordered[index + 1]?.address;
    check(end !== undefined, 'noise_native_function_end');
    const reachable = new Map(), pending = [fn.address], provisional = new Map();
    let resolved;
    do {
      while (pending.length) {
      check(++visited <= 250000, 'noise_native_instruction_bound');
      const address = pending.pop();
      if (reachable.has(address)) continue;
      if (!(address >= fn.address && address < end)) throw Object.assign(Error('noise_native_branch_outside_function'), { symbol: fn.symbol, address, end });
      if (!rows.has(address)) {
        check(++ranges <= 2048, 'noise_native_decode_bound');
        const decoded = decode(await readRange(address, Math.min(end, address + 4096)));
        if (!decoded.has(address)) throw Object.assign(Error('noise_native_instruction_missing'), { symbol: fn.symbol, address, end });
        for (const [at, instruction] of decoded) if (!rows.has(at)) rows.set(at, instruction);
      }
      const instruction = rows.get(address);
      check(instruction.bytes >= 2 && instruction.bytes <= 3, 'noise_native_instruction_size');
      reachable.set(address, instruction);
      if (/^(?:ret\w*(?:\.n)?|ill|break\w*)$/u.test(instruction.op)) continue;
      if (instruction.op === 'jx') { unresolvedJumps.push({ symbol: fn.symbol, address }); continue; }
      if (/^(?:j|b\w*(?:\.n)?|loop(?:nez|gtz)?)$/u.test(instruction.op)) {
        const destination = branch(instruction);
        check(destination !== null, 'noise_native_branch_unknown'); pending.push(destination);
      }
      if (instruction.op !== 'j') {
        const next = address + instruction.bytes;
        if (terminalCall(instruction, reachable)) {
          reachable.set(address, { ...instruction, terminal: true }); provisional.set(address, next);
        }
        else pending.push(next);
      }
      }
      resolved = { ...fn, instructions: [...reachable.values()].sort((a, b) => a.address - b.address) };
      const edges = new Map();
      for (const edge of noiseNativeCalls(resolved)) edges.set(edge.call_address, [...(edges.get(edge.call_address) ?? []), edge.target]);
      for (const [address, next] of provisional) {
        const candidates = edges.get(address);
        if (candidates?.length && candidates.every(target => noReturn(functions.get(target)?.symbol ?? ''))) continue;
        // A bypassing branch invalidates the apparent lexical panic load. Resume its tail.
        provisional.delete(address); reachable.set(address, { ...reachable.get(address), terminal: false }); pending.push(next);
      }
    } while (pending.length);
    result.set(fn.address, resolved);
    for (const edge of noiseNativeCalls(resolved)) {
      const callee = functions.get(edge.target);
      if (callee && selectedNoiseSymbol(callee.symbol)) candidates.push(callee);
    }
  }
  return { functions: result, supplementalRanges: ranges, unresolvedJumps };
}
