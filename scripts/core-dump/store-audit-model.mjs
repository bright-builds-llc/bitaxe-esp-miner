const check = (ok, code) => { if (!ok) throw Error(code); };
export const STORE_SEAMS = ['store', 'write_init', 'write_prepare', 'write_start', 'write_end'];
export const iram = value => value >= 0x40370000 && value < 0x403e0000;
const dram = value => value >= 0x3fc80000 && value < 0x3fd00000;
const rtc = value => (value >= 0x50000000 && value < 0x50002000) || (value >= 0x600fe000 && value < 0x60100000);
const register = value => /^a\d+$/u.test(value) ? Number(value.slice(1)) : null;
const number = value => /^-?(?:0x[0-9a-f]+|\d+)$/u.test(value) ? Number(value) : null;
const stack = value => value && typeof value === 'object' && Object.hasOwn(value, 'stack');
const add = (left, right) => typeof right !== 'number' ? null : typeof left === 'number' ? (left + right) >>> 0 :
  stack(left) ? { stack: left.stack + right } : left?.sdkLength !== undefined ? { sdkLength: left.sdkLength + right } : left?.sdkData !== undefined ? { sdkData: left.sdkData + right } : null;
const args = row => row.args.split(',').map(value => value.trim());
export function instructions(text) {
  return text.split('\n').flatMap(line => {
    const match = /^\s*([0-9a-f]+):\s+[0-9a-f]+\s+([a-z][a-z0-9_.]*)\s*(.*)$/u.exec(line);
    return match ? [{ address: parseInt(match[1], 16), op: match[2], args: match[3] }] : [];
  });
}
export function elfView(bytes) {
  check(bytes.length >= 52 && bytes.subarray(0, 7).equals(Buffer.from([127,69,76,70,1,1,1])) && bytes.readUInt16LE(18) === 94, 'store_audit_elf');
  const base = bytes.readUInt32LE(28), width = bytes.readUInt16LE(42), count = bytes.readUInt16LE(44);
  check(width === 32 && count < 256 && base + width * count <= bytes.length, 'store_audit_segments');
  const segments = Array.from({ length: count }, (_, i) => {
    const offset = base + i * width;
    return { type: bytes.readUInt32LE(offset), file: bytes.readUInt32LE(offset + 4), address: bytes.readUInt32LE(offset + 8), size: bytes.readUInt32LE(offset + 16) };
  });
  const sectionBase = bytes.readUInt32LE(32), sectionWidth = bytes.readUInt16LE(46), sectionCount = bytes.readUInt16LE(48);
  check(sectionWidth === 40 && sectionCount < 1024 && sectionBase + sectionWidth * sectionCount <= bytes.length, 'store_audit_sections');
  const namesIndex = bytes.readUInt16LE(50); check(namesIndex < sectionCount, 'store_audit_section_names');
  const namesHeader = sectionBase + sectionWidth * namesIndex, namesOffset = bytes.readUInt32LE(namesHeader + 16), namesSize = bytes.readUInt32LE(namesHeader + 20);
  check(namesOffset + namesSize <= bytes.length, 'store_audit_section_names');
  const sections = Array.from({ length: sectionCount }, (_, i) => {
    const at = sectionBase + i * sectionWidth;
    const nameOffset = bytes.readUInt32LE(at); check(nameOffset < namesSize, 'store_audit_section_names');
    const end = bytes.indexOf(0, namesOffset + nameOffset); check(end >= 0 && end < namesOffset + namesSize, 'store_audit_section_names');
    return { name: bytes.toString('ascii', namesOffset + nameOffset, end), type: bytes.readUInt32LE(at + 4), address: bytes.readUInt32LE(at + 12), size: bytes.readUInt32LE(at + 20) };
  });
  return {
    word(address) {
      const segment = segments.find(row => row.type === 1 && address >= row.address && address + 4 <= row.address + row.size);
      check(segment && segment.file + address - segment.address + 4 <= bytes.length, 'store_audit_literal');
      return bytes.readUInt32LE(segment.file + address - segment.address);
    },
    noLoad(symbol) { return sections.some(row => row.name === '.rtc_noinit' && row.type === 8 && symbol.address >= row.address && symbol.address + symbol.size <= row.address + row.size); },
  };
}
export function symbols(text) {
  return text.split('\n').flatMap(line => {
    const match = /^([0-9a-f]+)(?:\s+([0-9a-f]+))?\s+([A-Za-z])\s+(\S+)$/u.exec(line);
    return match ? [{ name: match[4], binding: match[3], address: parseInt(match[1], 16), size: match[2] ? parseInt(match[2], 16) : 0 }] : [];
  });
}
function frame(fn) {
  const entries = fn.code.filter(row => row.op === 'entry');
  check(entries.length === 1 && entries[0] === fn.code[0], 'store_audit_entry');
  const parsed = /^a1,\s*(0x[0-9a-f]+|\d+)$/u.exec(entries[0].args), size = parsed ? Number(parsed[1]) : NaN;
  check(Number.isSafeInteger(size) && size >= 16 && size <= 256 && size % 16 === 0, 'store_audit_frame'); return size;
}
const target = value => { const match = /^([0-9a-f]+)(?:\s|$)/u.exec(value); return match ? parseInt(match[1], 16) : null; };
const writes = op => !/^(?:entry|s(?:8|16|32)i(?:\.n)?|b\w*(?:\.n)?|j|retw(?:\.n)?|callx?(?:0|4|8|12)|memw|nop(?:\.n)?)$/u.test(op);
function branchDecision(row, registers) {
  const p = args(row); let left = registers[register(p[0])], right = register(p[1]) !== null ? registers[register(p[1])] : number(p[1]);
  if (stack(left) && stack(right)) { left = left.stack; right = right.stack; }
  if (/^b(?:eqz|nez)(?:\.n)?$/u.test(row.op) && typeof left === 'number') return row.op.includes('nez') ? left !== 0 : left === 0;
  if (typeof left !== 'number' || typeof right !== 'number') return null;
  if (/^beq(?:i)?$/u.test(row.op)) return left === right;
  if (/^bne(?:i)?$/u.test(row.op)) return left !== right;
  if (/^bltu(?:i)?$/u.test(row.op)) return (left >>> 0) < (right >>> 0);
  if (/^bgeu(?:i)?$/u.test(row.op)) return (left >>> 0) >= (right >>> 0);
  if (row.op === 'blti') return (left | 0) < right;
  if (row.op === 'bgei') return (left | 0) >= right;
  return null;
}

/** Context-sensitive bounded scalar execution proves own stores, not SDK internals. */
function analyze(fn, initial, context, depth = 0) {
  check(depth < 3, 'store_audit_call_depth');
  const frameBytes = frame(fn), code = fn.code, positions = new Map(code.map((row, i) => [row.address, i]));
  check(code.length <= 2048 && iram(fn.address) && iram(fn.address + fn.size - 1), 'store_audit_iram');
  const regs = Array(16).fill(null); regs[1] = { stack: 0 };
  initial.forEach((value, i) => { regs[i + 2] = value; });
  const pending = [{ index: 0, regs, slots: new Map(), realCalls: 0, shift: null }], seen = new Set();
  let steps = 0, returns = 0, stores = 0, peak = frameBytes;
  const callSites = new Set();
  while (pending.length) {
    check(++steps <= 50000, 'store_audit_execution_bound');
    const state = pending.pop(), row = code[state.index], p = args(row), incoming = state.regs;
    const key = JSON.stringify([state.index, incoming, [...state.slots], state.realCalls, state.shift]);
    if (seen.has(key)) continue; seen.add(key);
    const next = { ...state, regs: [...incoming], slots: new Map(state.slots) };
    const dst = register(p[0]);
    check(/^(?:entry|l32r|mov(?:i)?(?:\.n)?|memw|s32i(?:\.n)?|l32i(?:\.n)?|addi?(?:\.n)?|addmi|addx[248]|sub(?:\.n)?|and|or|xor|not|neg|slli|srli|ssai|ssl|ssr|sll|srl|src|extui|b(?:eqz|nez|eq|ne|ltu|geu|lti|gei|eqi|nei|ltui|geui)(?:\.n)?|j|retw(?:\.n)?|callx?8|nop(?:\.n)?)$/u.test(row.op), 'store_audit_instruction');
    if (writes(row.op) && dst !== null) { check(dst !== 1, 'store_audit_dynamic_stack'); next.regs[dst] = null; }
    if (row.op === 'l32r') {
      const literal = target(p[1]); check(literal !== null && iram(literal), 'store_audit_literal_iram'); next.regs[dst] = context.elf.word(literal);
    } else if (/^movi(?:\.n)?$/u.test(row.op)) next.regs[dst] = number(p[1]);
    else if (/^mov(?:\.n)?$/u.test(row.op)) next.regs[dst] = incoming[register(p[1])];
    else if (/^(?:addi(?:\.n)?|addmi)$/u.test(row.op)) next.regs[dst] = add(incoming[register(p[1])], number(p[2]));
    else if (/^add(?:\.n)?$/u.test(row.op)) next.regs[dst] = add(incoming[register(p[1])], incoming[register(p[2])]) ?? add(incoming[register(p[2])], incoming[register(p[1])]);
    else if (/^addx[248]$/u.test(row.op) && typeof incoming[register(p[1])] === 'number') next.regs[dst] = add(incoming[register(p[2])], incoming[register(p[1])] * Number(row.op.at(-1)));
    else if (row.op === 'ssai') next.shift = number(p[0]);
    else if (row.op === 'ssl' || row.op === 'ssr') next.shift = incoming[register(p[0])];
    else if ((row.op === 'sll' || row.op === 'srl') && typeof incoming[register(p[1])] === 'number' && typeof state.shift === 'number') next.regs[dst] = (row.op === 'sll' ? incoming[register(p[1])] << state.shift : incoming[register(p[1])] >>> state.shift) >>> 0;
    else if (/^sub(?:\.n)?$/u.test(row.op) && typeof incoming[register(p[2])] === 'number') next.regs[dst] = add(incoming[register(p[1])], -incoming[register(p[2])]);
    else if (['and','or','xor'].includes(row.op)) {
      const a = incoming[register(p[1])], b = incoming[register(p[2])];
      if (typeof a === 'number' && typeof b === 'number') next.regs[dst] = (row.op === 'and' ? a & b : row.op === 'or' ? a | b : a ^ b) >>> 0;
    } else if (['slli','srli'].includes(row.op) && typeof incoming[register(p[1])] === 'number') {
      const a = incoming[register(p[1])], shift = number(p[2]); next.regs[dst] = (row.op === 'slli' ? a << shift : a >>> shift) >>> 0;
    } else if (row.op === 'extui' && typeof incoming[register(p[1])] === 'number') next.regs[dst] = (incoming[register(p[1])] >>> number(p[2])) & (2 ** number(p[3]) - 1);
    else if (/^[ls]32i(?:\.n)?$/u.test(row.op)) {
      const address = add(incoming[register(p[1])], number(p[2]));
      if (row.op.startsWith('s')) {
        check(stack(address) ? address.stack >= 0 && address.stack + 4 <= frameBytes && address.stack % 4 === 0 : typeof address === 'number' &&
          context.data.some(symbol => address >= symbol.address && address + 4 <= symbol.address + symbol.size && address % 4 === 0), 'store_audit_write_bound');
        stores++; if (stack(address)) next.slots.set(address.stack, incoming[dst]);
      } else {
        if (!(stack(address) ? address.stack >= 0 && address.stack + 4 <= frameBytes && address.stack % 4 === 0 :
          depth === 0 && fn.name === '__wrap_esp_core_dump_write_prepare' && address?.sdkLength === 0 ? true : typeof address === 'number' &&
          context.data.some(symbol => address >= symbol.address && address + 4 <= symbol.address + symbol.size && address % 4 === 0))) {
          throw Object.assign(Error('store_audit_read_bound'), { symbol: fn.name, instruction: row.address, pointer: address, operands: row.args });
        }
        next.regs[dst] = stack(address) ? state.slots.get(address.stack) ?? null : null;
      }
    }
    if (/^callx?8$/u.test(row.op)) {
      const called = row.op === 'callx8' ? incoming[register(p[0])] : target(p[0]);
      check(typeof called === 'number', 'store_audit_unknown_call'); callSites.add(row.address);
      if (called === context.real) {
        if (['__wrap_esp_core_dump_write_prepare', '__wrap_esp_core_dump_write_start', '__wrap_esp_core_dump_write_end'].includes(fn.name))
          check(incoming[10]?.sdkData === 0, 'store_audit_delegate_arguments');
        if (fn.name === '__wrap_esp_core_dump_write_prepare') check(incoming[11]?.sdkLength === 0, 'store_audit_delegate_arguments');
        next.realCalls++; check(next.realCalls <= 1, 'store_audit_duplicate_delegate');
      }
      else {
        check(called === context.helper.address, 'store_audit_foreign_call');
        const child = analyze(context.helper, incoming.slice(10, 14), { ...context, real: null }, depth + 1); peak = Math.max(peak, frameBytes + child.peak);
      }
      for (let i = 8; i < 16; i++) next.regs[i] = null;
      if (called === context.real) next.regs[10] = { sdkResult: true };
    }
    if (/^retw(?:\.n)?$/u.test(row.op)) { check(state.realCalls === (context.real === null ? 0 : 1), 'store_audit_delegate_path');
      if (context.real !== null) check(state.regs[2]?.sdkResult === true, 'store_audit_delegate_result');
      returns++; continue; }
    const successors = [];
    if (row.op === 'j' || row.op.startsWith('b')) {
      const destination = positions.get(target(p.at(-1))); check(destination !== undefined, 'store_audit_branch');
      const decision = row.op === 'j' ? true : branchDecision(row, incoming);
      if (decision !== false) successors.push(destination);
      if (decision !== true) successors.push(state.index + 1);
    } else successors.push(state.index + 1);
    for (const index of successors) {
      check(index < code.length, 'store_audit_fallthrough');
      if (index <= state.index) check(!seen.has(JSON.stringify([index, next.regs, [...next.slots], next.realCalls, next.shift])), 'store_audit_unbounded_loop');
      pending.push({ ...next, index });
    }
  }
  check(returns > 0, 'store_audit_no_return'); return { peak, stores, instructions: code.length, callSites: [...callSites] };
}

export function auditStore({ elf, entries, functions, sdkCalls }) {
  const one = name => { const rows = entries.filter(row => row.name === name); check(rows.length === 1, 'store_audit_symbol'); return rows[0]; };
  const rtcReceipt = one('BITAXE_CORE_DUMP_RTC'), current = one('BITAXE_CORE_DUMP_CURRENT'), helper = functions.get('bitaxe_core_dump_receipt_update');
  check(rtcReceipt.size === 80 && rtc(rtcReceipt.address) && rtc(rtcReceipt.address + 79) && elf.noLoad(rtcReceipt), 'store_audit_rtc');
  check(current.size === 80 && dram(current.address) && dram(current.address + 79), 'store_audit_current');
  check(helper, 'store_audit_helper');
  const results = [];
  for (const seam of STORE_SEAMS) {
    const real = one(`esp_core_dump_${seam}`), wrapper = functions.get(`__wrap_esp_core_dump_${seam}`);
    check(wrapper, 'store_audit_wrapper');
    const caller = seam === 'store' ? 'esp_core_dump_write_elf_and_check' : 'esp_core_dump_store';
    const routes = sdkCalls.get(caller) ?? [];
    check(routes.some(call => call.target === wrapper.address) && !routes.some(call => call.target === real.address), 'store_audit_sdk_route');
    const initial = seam === 'write_prepare' ? [{ sdkData: 0 }, { sdkLength: 0 }, null, null] :
      ['write_start', 'write_end'].includes(seam) ? [{ sdkData: 0 }, null, null, null] : [null, null, null, null];
    const result = analyze(wrapper, initial, { elf, helper, real: real.address, data: [rtcReceipt, current] });
    results.push({ seam, wrapper: wrapper.address.toString(16), real: real.address.toString(16), wrapper_frame_bytes: frame(wrapper), ...result });
  }
  const outer = results.find(row => row.seam === 'store');
  const peak = Math.max(outer.peak, ...results.filter(row => row.seam !== 'store').map(row => outer.wrapper_frame_bytes + row.peak));
  check(peak <= 256, 'store_audit_added_stack');
  return { schema: 'str005-native-core-store-audit-v1', wrappers_iram: true, receipt_rtc_noinit: true, current_metadata_internal: true,
    sdk_routes_wrapped: true, real_calls_preserved: true, diagnostic_call_closure: true, bounded_diagnostic_writes: true,
    max_added_stack_bytes: peak, added_stack_budget_bytes: 256, hardware_verified: false, seams: results };
}

/** Static scheduled initializer routing, not runtime success of the void initializer. */
export function auditBootInitializer(elf, entries, calls) {
  const one = name => { const found = entries.filter(row => row.name === name); check(found.length === 1, 'store_audit_boot_symbol'); return found[0]; };
  const record = one('esp_system_init_fn_init_coredump'), thunk = one('__esp_system_init_fn_init_coredump'), init = one('esp_core_dump_init');
  const start = one('_esp_system_init_fn_array_start'), end = one('_esp_system_init_fn_array_end');
  check(init.binding === 'T' && record.size === 8 && record.address >= start.address && record.address + 8 <= end.address &&
    elf.word(record.address) === thunk.address && elf.word(record.address + 4) === 0x00010001 &&
    calls.some(row => row.target === init.address), 'store_audit_boot_route');
  return { normal_boot_initializer_linked: true, normal_boot_init_runtime_verified: false };
}
