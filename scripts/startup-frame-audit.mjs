// Startup frame audit: the 16 KiB main task runs startup and then the Worker trust parse on one stack.
// Each budgeted function's own frame (its `entry a1, N` size) must stay within its bound, and large
// startup owners must stay standalone functions so their frames are released before that parse.

const FUNCTION = /^([0-9a-f]+) <(.+)>:$/u;
const INSTRUCTION = /^\s*[0-9a-f]+:\s+(?:[0-9a-f]{2,8}\s+)*([a-z][a-z0-9.]*)\s*(.*)$/u;
const ENTRY = /^a1,\s*(0x[0-9a-f]+|\d+)$/u;

/** Each function's own frame size from its windowed `entry` prologue (null for call0 routines). */
export function frameSizes(disassembly) {
  const frames = new Map();
  let maybeCurrent = null;
  for (const line of disassembly.split('\n')) {
    const header = FUNCTION.exec(line);
    if (header) { maybeCurrent = header[2]; continue; }
    if (maybeCurrent === null) continue;
    const instruction = INSTRUCTION.exec(line);
    if (!instruction) continue;
    const size = instruction[1] === 'entry' ? ENTRY.exec(instruction[2].trim()) : null;
    if (!frames.has(maybeCurrent)) frames.set(maybeCurrent, size ? Number(size[1]) : null);
    maybeCurrent = null;
  }
  return frames;
}

/** Passes only when every budgeted frame is present and within bounds and every owner is standalone. */
export function auditStartupFrames(disassembly, budget) {
  const frames = frameSizes(disassembly);
  const measured = budget.frames.map(({ symbol, max_bytes }) => ({ symbol, max_bytes, bytes: frames.get(symbol) ?? null }));
  const violations = [
    ...measured.filter(row => row.bytes === null || row.bytes > row.max_bytes)
      .map(row => ({ symbol: row.symbol, kind: row.bytes === null ? 'frame_missing' : 'frame_over_budget' })),
    ...budget.standalone.filter(({ symbol }) => !frames.has(symbol)).map(({ symbol }) => ({ symbol, kind: 'inlined' })),
  ];
  return { schema: 'startup-frame-audit-v1', frames: measured, violations,
    result: violations.length === 0 ? 'startup_frames_within_budget' : 'blocked' };
}
