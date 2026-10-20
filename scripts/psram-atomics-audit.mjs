// Rust code must never use the S32C1I instruction (ADR-0039). It is not atomic on PSRAM, where
// Rust heap objects live, so Rust atomics must go through ESP-IDF's libcalls instead. Only the
// native functions below may perform a hardware compare-exchange. ESP-IDF's ones handle PSRAM or
// touch internal RAM only; the `bitaxe_*` ones are the firmware's C fault recorders, whose own
// audit checks them (`fault-provenance-audit.mjs`). Any other function, whatever its symbol (a
// `#[no_mangle]` Rust function looks native), fails.

const ALLOWED_NATIVE = new Set(['esp_cpu_compare_and_set', '_xt_coproc_exc', '_xt_coproc_release',
  'bitaxe_allocation_failure_record', 'bitaxe_allocation_recoverable_begin', 'bitaxe_fault_owner_begin']);
/** ESP-IDF's libcall bodies for internal-RAM addresses. */
const ALLOWED_NATIVE_PREFIXES = ['__atomic_s32c1i_', '__sync_s32c1i_'];

/** Whether a function may perform a hardware compare-exchange. */
export const allowedHardwareAtomic = (symbol) =>
  ALLOWED_NATIVE.has(symbol) || ALLOWED_NATIVE_PREFIXES.some((prefix) => symbol.startsWith(prefix));

/** A compare-exchange writes SCOMPARE1 at most this many instructions before its `s32c1i`. */
const SCOMPARE_WINDOW = 8;

/**
 * Each function in a non-demangled objdump disassembly that performs a hardware compare-exchange,
 * with its count. Only an `s32c1i` shortly after a `wsr.scompare1` counts: one without it is data,
 * such as a literal pool, that objdump decoded as an instruction.
 */
export function hardwareAtomicFunctions(disassembly) {
  const counts = new Map();
  let maybeSymbol = null, sinceScompare = Infinity;
  for (const line of disassembly.split('\n')) {
    const header = /^[0-9a-f]+ <(.+)>:$/u.exec(line);
    if (header) { maybeSymbol = header[1]; sinceScompare = Infinity; continue; }
    // The opcode follows the address and, unless objdump omits them, the raw bytes.
    const instruction = /^\s*[0-9a-f]+:\t(?:[0-9a-f ]+\t)?(\S+)/u.exec(line);
    if (maybeSymbol === null || instruction === null) continue;
    sinceScompare += 1;
    if (instruction[1] === 'wsr.scompare1') sinceScompare = 0;
    if (instruction[1] === 's32c1i' && sinceScompare <= SCOMPARE_WINDOW) counts.set(maybeSymbol, (counts.get(maybeSymbol) ?? 0) + 1);
  }
  return [...counts].map(([symbol, count]) => ({ symbol, count }));
}

/** The audit's verdict: clean only when every hardware compare-exchange is in an allowed native function. */
export function auditPsramAtomics(disassembly) {
  const functions = hardwareAtomicFunctions(disassembly);
  const unexpected = functions.filter((entry) => !allowedHardwareAtomic(entry.symbol));
  return {
    schema: 'psram-atomics-audit-v2',
    result: unexpected.length === 0 ? 'no_unexpected_hardware_atomics' : 'unexpected_hardware_atomics',
    unexpected_functions: unexpected.map((entry) => entry.symbol).sort(),
    allowed_native_functions: functions.length - unexpected.length,
  };
}
