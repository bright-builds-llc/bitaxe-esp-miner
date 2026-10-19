// Test fixtures in the pinned objdump's `-d -C` and `-t` formats, for the thread stack audits.
export const TRAMPOLINE = 'std::sys::backtrace::__rust_begin_short_backtrace';

const hex = (value) => value.toString(16).padStart(8, '0');

/**
 * Rows are `[name, frame bytes, callees]`. A repeated name is another monomorphized instance, as
 * every spawned closure gets its own trampoline. A callee is a name (its first instance) or a row
 * index. Calls are `l32r` plus `callx8` pairs, as the firmware's long calls are, and each body is
 * followed by a literal-pool word that decodes as a stray call, which the symbol sizes must drop.
 */
export function objdumpFixture(rows) {
  const addresses = rows.map((_, index) => 0x42000000 + index * 0x100);
  const firstByName = new Map();
  rows.forEach(([name], index) => { if (!firstByName.has(name)) firstByName.set(name, index); });
  const target = (callee) => (typeof callee === 'number' ? callee : firstByName.get(callee));
  let literal = 0x3f000000;
  const bodies = rows.map(([name, bytes, callees], index) => {
    let at = addresses[index];
    const lines = [`${hex(at)} <${name}>:`, `${hex(at)}:\t004136        \tentry\ta1, 0x${bytes.toString(16)}`];
    at += 3;
    for (const callee of callees) {
      const to = target(callee);
      lines.push(`${hex(at)}:\tebe081        \tl32r\ta8, ${hex(literal)} <_stext+0x${(literal & 0xfff).toString(16)}> ` +
        `(${hex(addresses[to])} <${rows[to][0]}>)`, `${hex(at + 3)}:\t0008e0        \tcallx8\ta8`);
      at += 6; literal += 4;
    }
    lines.push(`${hex(at)}:\tf01d      \tretw.n`);
    const size = at + 2 - addresses[index];
    // Pool words after the function that decode as a call to the last row.
    lines.push(`${hex(at + 2)}:\t0008e0        \tcallx8\ta8`,
      `${hex(at + 5)}:\t000005        \tcall8\t${hex(addresses.at(-1))} <${rows.at(-1)[0]}>`, '');
    return { lines, symbol: `${hex(addresses[index])} g     F .flash.text\t${hex(size)} ${name}` };
  });
  return { disassembly: bodies.flatMap((body) => body.lines).join('\n'),
    symbols: bodies.map((body) => body.symbol).join('\n') + '\n' };
}
