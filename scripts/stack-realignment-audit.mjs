// Stack realignment audit: the Xtensa LLVM backend realigns over-aligned frames with a plain
// write to a1 instead of `movsp`. An interrupt in that prologue makes the caller restore its
// registers from stale memory, so no project code may reach such a function except through an
// explicit, reviewed allowlist. Every esp toolchain from 1.88.0.0 to 1.99.0.0 is affected:
//   https://github.com/espressif/llvm-project/issues/140 (backend fix)
//   https://github.com/esp-rs/rust/issues/284 (Rust tracking)
//   https://github.com/pRizz/xtensa-movsp-realign-repro (reproducer)
// The firmware uses bitaxe_runtime::reply and bitaxe_runtime::queue instead of std channels.

const FUNCTION = /^([0-9a-f]+) <(.+)>:$/u;
const INSTRUCTION = /^\s*([0-9a-f]+):\s+(?:[0-9a-f]{2,8}\s+)+([a-z][a-z0-9.]*)\s*(.*)$/u;
const REALIGN = new Set(['add', 'add.n', 'sub', 'and']);
// The realignment sits in the prologue; later matches are literal-pool words decoded as code.
const PROLOGUE_INSTRUCTIONS = 8;
const DIRECT_CALL = /^call(?:0|4|8|12)$/u;
// Match the address only: demangled names may themselves contain `<` and `>`.
const TARGET = /\b([0-9a-f]{8})\s+</u;
// `l32r` prints the literal slot first and the loaded function in parentheses.
const LITERAL_TARGET = /\(([0-9a-f]{8})\s+</u;
// std's channel internals and their generic wrappers forward to the realigning constructor;
// the audit walks through them to the first project-owned caller.
const TRANSPARENT = /^(?:std::sync::(?:mpsc|mpmc)::|<std::sync::(?:mpsc|mpmc)::)/u;

/** Function records with their stack-pointer writes and referenced call targets. */
export function parseFunctions(disassembly) {
  const functions = new Map();
  let current = null;
  for (const line of disassembly.split('\n')) {
    const header = FUNCTION.exec(line);
    if (header) {
      current = { address: header[1], name: header[2], windowed: false, realigns: false, references: new Set(), instructions: 0 };
      functions.set(current.address, current);
      continue;
    }
    const instruction = current && INSTRUCTION.exec(line);
    if (!instruction) continue;
    const [, , mnemonic, operands] = instruction;
    const index = current.instructions++;
    if (index === 0) current.windowed = mnemonic === 'entry';
    // Only windowed (`entry`) functions must move a1 with `movsp`; call0 port routines switch stacks by design.
    if (current.windowed && index < PROLOGUE_INSTRUCTIONS && REALIGN.has(mnemonic) && /^a1\s*,\s*a1\s*,/u.test(operands))
      current.realigns = true;
    const target = mnemonic === 'l32r' ? LITERAL_TARGET.exec(operands) : DIRECT_CALL.test(mnemonic) ? TARGET.exec(operands) : null;
    if (target) current.references.add(target[1]);
  }
  return functions;
}

/** Project-owned callers of realigning functions, walking through std channel wrappers. */
export function realignmentCallers(functions) {
  const callersOf = new Map();
  for (const fn of functions.values()) for (const target of fn.references) {
    if (!callersOf.has(target)) callersOf.set(target, new Set());
    callersOf.get(target).add(fn.address);
  }
  const realigning = [...functions.values()].filter(fn => fn.realigns);
  const owners = new Set(), seen = new Set(realigning.map(fn => fn.address));
  const queue = [...seen];
  while (queue.length) {
    for (const caller of callersOf.get(queue.shift()) ?? []) {
      if (seen.has(caller)) continue;
      seen.add(caller);
      const fn = functions.get(caller);
      if (TRANSPARENT.test(fn.name)) queue.push(caller);
      else owners.add(fn.name);
    }
  }
  return { realigning: realigning.map(fn => fn.name).sort(), callers: [...owners].sort() };
}

/** Passes only when every project-owned caller is on the reviewed allowlist (empty for the firmware). */
export function auditStackRealignment(disassembly, allowlist) {
  const { realigning, callers } = realignmentCallers(parseFunctions(disassembly));
  const allowed = new Set(allowlist.map(entry => entry.symbol));
  const unexpected = callers.filter(name => !allowed.has(name));
  return { schema: 'stack-realignment-audit-v1', realigning_functions: realigning.length, callers,
    unexpected_callers: unexpected, result: unexpected.length === 0 ? 'no_runtime_realignment_callers' : 'blocked' };
}
