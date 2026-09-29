import { noiseNativeCalls } from './noise-native-calls.mjs';
const check = (ok, code) => { if (!ok) throw Error(code); };
const iram = n => n >= 0x40370000 && n < 0x403e0000;
const dram = n => n >= 0x3fc88000 && n < 0x3fd00000;
export const ROOTS = ['bitaxe_capture_original_panic', '__wrap_esp_core_dump_check_task', '__wrap_esp_core_dump_port_set_crashed_tcb',
  'bitaxe_allocation_failure_record', 'bitaxe_fault_owner_begin', 'bitaxe_fault_owner_end', 'bitaxe_fault_command_begin', 'bitaxe_fault_enter_phase'];
const forbidden = /malloc|calloc|realloc|free$|printf|log_write|mutex|Semaphore|EnterCritical|ExitCritical|spinlock|panic_abort|assert|alloc_error|__atomic_.*_8/iu;
/** Native IRAM call closure and literal/data placement; no unknown indirect edge is permitted. */
export function auditFaultProvenance({ entries, functions, word, romLeaf }) {
  const one = name => { const rows = entries.filter(row => row.name === name); if (rows.length !== 1) throw Object.assign(Error('fault_audit_symbol'), { symbol: name }); return rows[0]; };
  const regionStart = one('_coredump_dram_start').address, regionEnd = one('_coredump_dram_end').address;
  for (const [name, size] of [['BITAXE_PANIC_FRAME_RECORD', 192], ['BITAXE_ALLOCATION_HISTORY', 1536]]) {
    const symbol = one(name); check(symbol.size === size && dram(symbol.address) && symbol.address >= regionStart && symbol.address + size <= regionEnd, 'fault_audit_capture_region');
  }
  for (const name of ['owners', 'legacy_allocation', 'legacy_context', 'legacy_source_lo', 'legacy_source_hi', 'legacy_claim', 'identity', 'snapshot', 'checks', 'capture_active']) {
    const item = one(name); check(item.size > 0 && dram(item.address) && dram(item.address + item.size - 1), 'fault_audit_state_dram');
  }
  const abi = one('BITAXE_FAULT_PROVENANCE_ABI'); check(abi.size === 128, 'fault_audit_abi');
  const expected = [0x50464131, 1, 48, 36, 112, 12, 0, 4, 8, 12, 24, 28, 32, 0, 4, 8, 12, 16, 80, 84, 0, 4, 8];
  check(expected.every((value, index) => word(abi.address + index * 4) === value) && word(abi.address + 28 * 4) === 65536 && word(abi.address + 31 * 4) === 0x50465231, 'fault_audit_abi');
  check(one('BITAXE_FAULT_COMPILED_SOURCE').size === 8, 'fault_audit_source');
  const delegates = new Map([['__wrap_esp_core_dump_check_task', 'esp_core_dump_check_task'], ['__wrap_esp_core_dump_port_set_crashed_tcb', 'esp_core_dump_port_set_crashed_tcb']]);
  const memo = new Map(), active = new Set();
  function visit(address) {
    if (memo.has(address)) return memo.get(address);
    if (romLeaf && address === romLeaf.address) { if (!(romLeaf.symbol === '_xtos_set_intlevel' && romLeaf.stackBytes === 16 && romLeaf.sdk_rom_selector_verified === true)) throw Error('fault_audit_rom_leaf'); return 16; }
    const fn = functions.get(address);
    const check = (ok, code) => { if (!ok) throw Object.assign(Error(code), { symbol: fn?.symbol ?? entries.find(row => row.address === address)?.name ?? 'unmapped' }); };
    check(!active.has(address), 'fault_audit_recursive_call'); active.add(address);
    check(fn && iram(address) && fn.instructions.length > 0 && fn.instructions.length <= 4096, 'fault_audit_iram');
    check(!forbidden.test(fn.symbol), 'fault_audit_forbidden_call');
    const instructions = fn.instructions; check(instructions.every(row => iram(row.address)), 'fault_audit_iram');
    const entry = instructions.find(row => row.op === 'entry'), frame = entry ? Number(entry.args.split(',')[1]) : 0;
    check(Number.isSafeInteger(frame) && frame <= 256 && frame % 16 === 0, 'fault_audit_frame');
    for (const row of instructions) {
      check(row.op !== 'jx', 'fault_audit_indirect_branch');
      check(!(row.args.split(',')[0].trim() === 'a1' && !/^(?:entry|s(?:8|16|32)i|b|j|ret|call|memw|loop)/u.test(row.op)), 'fault_audit_dynamic_stack');
      if (/^(?:j|b\w*(?:\.n)?|loop(?:nez|gtz)?)$/u.test(row.op)) {
        const target = /(?:^|,\s*)([a-f0-9]+)\s*</u.exec(row.args);
        if (!(target && instructions.some(item => item.address === parseInt(target[1], 16)))) throw Object.assign(Error('fault_audit_branch_escape'), { symbol: fn.symbol, opcode: row.op, targetSymbol: /<([^>]+)>/u.exec(row.args)?.[1] });
      }
      if (row.op === 'l32r') {
        const literal = /,\s*([a-f0-9]+)/u.exec(row.args); check(literal && iram(parseInt(literal[1], 16)), 'fault_audit_literal_iram');
        const value = word(parseInt(literal[1], 16));
        // Cached DROM/PSRAM pointer literals indicate unsafe tables or data on a fatal path.
        check(value < 0x3c000000 || value >= 0x3e000000, 'fault_audit_cached_data');
      }
    }
    const calls = noiseNativeCalls(fn, { compilerPrivateSpills: true });
    check(instructions.filter(row => /^callx?(?:0|4|8|12)$/u.test(row.op)).every(row => calls.some(call => call.call_address === row.address)), 'fault_audit_unknown_call');
    let nested = 0, delegated = 0;
    for (const call of calls) {
      check(call.target !== null && call.target !== undefined, 'fault_audit_unknown_call');
      const delegate = delegates.get(fn.symbol);
      if (delegate && call.target === one(delegate).address) { delegated++; continue; }
      try { nested = Math.max(nested, visit(call.target)); } catch (error) { error.ancestry = [fn.symbol, ...(error.ancestry ?? [])]; throw error; }
    }
    if (delegates.has(fn.symbol)) {
      if (delegated < 1) throw Object.assign(Error('fault_audit_sdk_delegate'), { symbol: fn.symbol, delegateCalls: delegated });
      const positions = new Map(instructions.map((row, index) => [row.address, index]));
      const sites = new Set(calls.filter(call => call.target === one(delegates.get(fn.symbol)).address).map(call => call.call_address));
      const pending = [[0, 0]], seen = new Set(); let exits = 0;
      while (pending.length) {
        const [index, count] = pending.pop(), row = instructions[index], nextCount = count + Number(sites.has(row.address));
        check(nextCount <= 1, 'fault_audit_sdk_delegate');
        const key = `${index}:${nextCount}`; if (seen.has(key)) continue; seen.add(key);
        if (/^ret/u.test(row.op)) { check(nextCount === 1, 'fault_audit_sdk_delegate'); exits++; continue; }
        if (/^(?:j|b\w*(?:\.n)?)$/u.test(row.op)) { const target = /(?:^|,\s*)([a-f0-9]+)\s*</u.exec(row.args); pending.push([positions.get(parseInt(target[1], 16)), nextCount]); }
        if (row.op !== 'j' && index + 1 < instructions.length) pending.push([index + 1, nextCount]);
      }
      check(exits > 0, 'fault_audit_sdk_delegate');
    }
    active.delete(address); const total = frame + nested; memo.set(address, total); return total;
  }
  const sdk = functions.get(one('esp_core_dump_get_task_snapshot').address); check(sdk, 'fault_audit_sdk_caller');
  const sdkCalls = noiseNativeCalls(sdk);
  for (const [wrapper, real] of delegates) check(sdkCalls.some(call => call.target === one(wrapper).address) && !sdkCalls.some(call => call.target === one(real).address), 'fault_audit_sdk_routing');
  const callback = functions.get(one('bitaxe_allocation_failure_record').address);
  const atomic = callback.instructions.filter(row => row.op === 's32c1i');
  check(atomic.length === 1 && callback.instructions.some(row => row.op === 'wsr.scompare1'), 'fault_audit_legacy_single_cas');
  for (const row of callback.instructions.filter(row => /^(?:j|b\w*(?:\.n)?)$/u.test(row.op))) {
    const target = /(?:^|,\s*)([a-f0-9]+)\s*</u.exec(row.args);
    if (target) check(!(parseInt(target[1], 16) <= atomic[0].address && row.address >= atomic[0].address && parseInt(target[1], 16) <= row.address), 'fault_audit_cas_retry');
  }
  const stack = ROOTS.map(name => { try { return visit(one(name).address); } catch (error) { error.root ??= name; throw error; } });
  const wrapper = functions.get(one('__wrap_esp_panic_handler').address);
  const wrapperFrame = Number(wrapper?.instructions[0]?.args.split(',')[1]);
  check(wrapper?.instructions[0]?.op === 'entry' && Number.isSafeInteger(wrapperFrame), 'fault_audit_cutoff_frame');
  const panicStack = wrapperFrame + stack[0], allocationStack = stack[3];
  check(panicStack <= 128 && Math.max(stack[1], stack[2]) <= 128, 'fault_audit_panic_stack_budget');
  check(allocationStack <= 256 && Math.max(...stack.slice(4)) <= 256, 'fault_audit_allocation_stack_budget');
  return { schema: 'str005-native-fault-provenance-audit-v1', native_abi_verified: true, captured_regions_verified: true,
    hot_call_closure_cache_safe: true, local_hot_functions_iram: true, exact_sdk_irq_rom_exception: Boolean(romLeaf), state_internal_dram: true, sdk_routes_wrappers: true, hot_literals_iram: true, cached_pointer_literals_absent: true, forbidden_calls_absent: true,
    unknown_calls_absent: true, legacy_hardware_cas_single_no_retry: true, panic_added_stack_bytes: panicStack, allocation_added_stack_bytes: allocationStack, max_selected_added_stack_bytes: Math.max(panicStack, ...stack), audited_functions: memo.size, hardware_verified: false };
}
