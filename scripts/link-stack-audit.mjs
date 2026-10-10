// Serial link RX thread stack audit: deepest resolved normal path from the link loop.
import { parseNoiseFrames } from './virtual-emulator/noise-stack-audit.mjs';
import { deepestNormalPath, MARGIN_BYTES, THREAD_ENTRY_ALLOWANCE_BYTES } from './control-stack-audit.mjs';

export const LINK_STACK_BYTES = 8192;
const LINK = 'bitaxe_firmware::bwg_worker_usb::link::run';
const CONTRACT = 'prepared_thread::spawn("bwg-serial-link", 8192,';

/**
 * Adds every function whose address a frame loads as a possible callee. The compiler
 * hoists loop call targets into stack slots and calls them through `callx8`, which
 * register tracking cannot follow; over-approximating keeps the bound conservative.
 */
export function withAddressTakenCallees(disassembly, frames) {
  const labels = [...disassembly.matchAll(/^([a-f0-9]+) <(.+)>:\s*$/gm)];
  for (let index = 0; index < labels.length; index++) {
    const frame = frames.byAddress.get(Number.parseInt(labels[index][1], 16));
    const body = disassembly.slice(labels[index].index + labels[index][0].length, labels[index + 1]?.index ?? disassembly.length);
    for (const match of body.matchAll(/\sl32r\s+a\d+,\s*[a-f0-9]+ <[^\n]*\(([a-f0-9]+) <[^\n]*>\)\s*$/gm)) {
      const target = Number.parseInt(match[1], 16);
      if (frames.byAddress.has(target) && !frame.calls.includes(target)) frame.calls.push(target);
    }
  }
  return frames;
}

/** Selected lower bound only: indirect calls and excluded edges are not bounded. */
export function auditLinkStack(disassembly, preparedLinkSource) {
  if (!preparedLinkSource.includes(CONTRACT)) throw Error('link_stack_contract');
  const frames = withAddressTakenCallees(disassembly, parseNoiseFrames(disassembly));
  const deepest = deepestNormalPath(frames, LINK);
  const total = deepest.bytes + THREAD_ENTRY_ALLOWANCE_BYTES, budget = LINK_STACK_BYTES - MARGIN_BYTES;
  return { schema: 'link-stack-audit-v1', link_stack_bytes: LINK_STACK_BYTES, budget_bytes: budget,
    thread_entry_allowance_bytes: THREAD_ENTRY_ALLOWANCE_BYTES, deepest_normal_path_bytes: total,
    headroom_bytes: budget - total, deepest_symbols: deepest.symbols, unresolved_targets: deepest.unresolved_targets,
    complete_callgraph_bound: false, excluded_edges: 'panic_abort_logging', address_taken_callees: true,
    result: total <= budget ? 'selected_path_with_headroom' : 'blocked' };
}
