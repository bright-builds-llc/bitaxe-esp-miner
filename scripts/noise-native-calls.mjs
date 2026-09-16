const check = (condition, code) => { if (!condition) throw Error(code); };
const target = instruction => {
  const match = /(?:^|,\s*)([0-9a-f]+)\s+</u.exec(instruction.args);
  return match ? parseInt(match[1], 16) : null;
};
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const targets = value => typeof value === 'number' ? [value] : value?.targets ?? [];
const stackOrigin = value => value?.stack ?? value?.maybeStack;
const join = (a, b) => {
  if (same(a, b)) return a;
  const stack = [stackOrigin(a), stackOrigin(b)].filter(value => value !== undefined);
  if (stack.length) return { maybeStack: Math.min(...stack) };
  const left = targets(a), right = targets(b);
  if (!left.length || !right.length) return null;
  const union = [...new Set([...left, ...right])].sort((x, y) => x - y);
  return union.length <= 8 ? { targets: union } : null;
};
const copy = state => ({ registers: [...state.registers], slots: new Map(state.slots) });
const merge = (a, b) => ({
  registers: a.registers.map((value, index) => join(value, b.registers[index])),
  slots: new Map([...new Set([...a.slots.keys(), ...b.slots.keys()])]
    .map(offset => [offset, join(a.slots.get(offset), b.slots.get(offset))])),
});
const changed = (a, b) => a.registers.some((value, index) => !same(value, b.registers[index]))
  || a.slots.size !== b.slots.size || [...a.slots].some(([offset, value]) => !same(value, b.slots.get(offset)));
const register = value => /^a(\d+)$/u.test(value) ? Number(value.slice(1)) : null;
const writesRegister = op => !/^(?:s8i|s16i|s32i(?:\.n)?|ssi|ssx|b\w*(?:\.n)?|j|jx|call\w*|ret\w*|nop\w*|loop\w*|memw|isync|rsync|esync|dsync|waiti|ill|break\w*)$/u.test(op);

/** Selected compiled call edges, including fixed local spills; unknown targets stay unknown.
 * Uses normal compiler stack ownership: calls cannot mutate unexposed spill slots.
 * Explicit stack-address arguments invalidate the exposed tail before return.
 */
function analyze(fn) {
  const code = fn.instructions, positions = new Map(code.map((instruction, index) => [instruction.address, index]));
  if (!code.length) return { calls: [], instructions: [] };
  const initial = { registers: Array(16).fill(null), slots: new Map() }; initial.registers[1] = { stack: 0 };
  const states = new Map([[0, initial]]), queue = [0], edges = new Map(), loops = new Map();
  let visits = 0;
  while (queue.length) {
    check(++visits < 1000000, 'noise_native_dataflow_bound');
    const index = queue.shift(), instruction = code[index], incoming = states.get(index), outgoing = copy(incoming);
    const args = instruction.args.split(',').map(value => value.trim()), dest = register(args[0]);
    if (writesRegister(instruction.op) && dest !== null && instruction.op !== 'entry') {
      const tainted = args.slice(1).some(arg => stackOrigin(incoming.registers[register(arg)]) !== undefined);
      outgoing.registers[dest] = tainted ? { maybeStack: 0 } : null;
    }
    if (instruction.op === 'l32r') {
      const literal = /^a(\d+),.*\(([0-9a-f]+) <.+>\)\s*$/u.exec(instruction.args);
      if (literal) outgoing.registers[Number(literal[1])] = parseInt(literal[2], 16);
    } else if (/^mov(?:\.n)?$/u.test(instruction.op)) {
      outgoing.registers[dest] = incoming.registers[register(args[1])] ?? null;
    } else if (/^movi(?:\.n)?$/u.test(instruction.op)) {
      const value = Number(args[1]); if (Number.isSafeInteger(value)) outgoing.registers[dest] = value;
    } else if (/^(?:addi(?:\.n)?|addmi)$/u.test(instruction.op)) {
      const value = incoming.registers[register(args[1])], amount = Number(args[2]);
      if (Number.isSafeInteger(amount)) outgoing.registers[dest] = value?.stack !== undefined
        ? { stack: value.stack + amount } : stackOrigin(value) !== undefined ? { maybeStack: 0 } : typeof value === 'number' ? value + amount : null;
    } else if (/^add(?:\.n)?$/u.test(instruction.op)) {
      const left = incoming.registers[register(args[1])], right = incoming.registers[register(args[2])];
      if (left?.stack !== undefined && typeof right === 'number') outgoing.registers[dest] = { stack: left.stack + right };
      else if (right?.stack !== undefined && typeof left === 'number') outgoing.registers[dest] = { stack: right.stack + left };
    }
    if (/^(?:s(?:8|16|32)|l(?:8u|16u|32))i(?:\.n)?$/u.test(instruction.op)) {
      const base = incoming.registers[register(args[1])], offset = Number(args[2]);
      const location = base?.stack !== undefined && Number.isSafeInteger(offset) ? base.stack + offset : null;
      if (instruction.op.startsWith('s')) {
        if (location === null) outgoing.slots.clear();
        else {
          const width = Number(/\d+/u.exec(instruction.op)[0]) / 8;
          for (const key of outgoing.slots.keys()) if (key < location + width && key + 4 > location) outgoing.slots.delete(key);
          if (width === 4) outgoing.slots.set(location, incoming.registers[dest]);
        }
      } else {
        outgoing.registers[dest] = /^l32i/u.test(instruction.op) && location !== null ? incoming.slots.get(location) ?? null : null;
      }
    }
    // Conditional/unsupported stores cannot establish which previous bytes survived.
    if (/^s/u.test(instruction.op) && !/^s(?:8|16|32)i(?:\.n)?$/u.test(instruction.op)
      && !/^(?:sub(?:\.n)?|sll\w*|srl\w*|sra\w*|sext|ssl|ssr|src|salt\w*|slli|srli|srai)$/u.test(instruction.op)) outgoing.slots.clear();
    if (/^callx?(?:0|4|8|12)$/u.test(instruction.op)) {
      const value = instruction.op.startsWith('callx') ? incoming.registers[dest] : target(instruction);
      if (targets(value).length) edges.set(instruction.address, targets(value).map(target => ({ target, call_address: instruction.address })));
      else edges.delete(instruction.address);
      const first = instruction.op.endsWith('8') ? 8 : 0;
      for (let r = first; r < 16; r++) {
        const pointer = incoming.registers[r];
        const escaped = stackOrigin(pointer);
        if (escaped !== undefined) for (const offset of outgoing.slots.keys()) if (offset >= escaped) outgoing.slots.delete(offset);
        if (r !== 1) outgoing.registers[r] = null;
      }
    }
    if (/^loop(?:nez|gtz)?$/u.test(instruction.op)) {
      const end = positions.get(target(instruction));
      check(end !== undefined && end > index + 1, 'noise_native_loop_unknown');
      loops.set(end - 1, index + 1);
    }
    const successors = [];
    if (!instruction.terminal && !/^(?:ret\w*(?:\.n)?|jx|ill|break\w*)$/u.test(instruction.op)) {
      if (instruction.op !== 'j' && index + 1 < code.length) successors.push(index + 1);
      if (/^(?:j|b\w*(?:\.n)?|loop(?:nez|gtz)?)$/u.test(instruction.op)) {
        const branch = positions.get(target(instruction)); if (branch !== undefined) successors.push(branch);
      }
      if (loops.has(index)) successors.push(loops.get(index));
    }
    for (const next of successors) {
      const prior = states.get(next), result = prior ? merge(prior, outgoing) : copy(outgoing);
      if (!prior || changed(prior, result)) { states.set(next, result); queue.push(next); }
    }
  }
  return { calls: [...edges.values()].flat(), instructions: code.filter((_instruction, index) => states.has(index)) };
}

export const noiseNativeCalls = fn => analyze(fn).calls;
export const noiseNativeInstructions = fn => analyze(fn).instructions;
