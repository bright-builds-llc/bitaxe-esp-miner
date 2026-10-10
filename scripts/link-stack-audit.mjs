// Serial link RX thread stack audit: deepest resolved normal path from the link loop.
import { parseNoiseFrames } from './virtual-emulator/noise-stack-audit.mjs';
import { deepestNormalPath, MARGIN_BYTES, THREAD_ENTRY_ALLOWANCE_BYTES } from './control-stack-audit.mjs';

export const LINK_STACK_BYTES = 8192;
const LINK = 'bitaxe_firmware::bwg_worker_usb::link::run';
const CONTRACT = 'prepared_thread::spawn("bwg-serial-link", 8192,';

/** Selected lower bound only: indirect calls and excluded edges are not bounded. */
export function auditLinkStack(disassembly, preparedLinkSource) {
  if (!preparedLinkSource.includes(CONTRACT)) throw Error('link_stack_contract');
  const deepest = deepestNormalPath(parseNoiseFrames(disassembly), LINK);
  const total = deepest.bytes + THREAD_ENTRY_ALLOWANCE_BYTES, budget = LINK_STACK_BYTES - MARGIN_BYTES;
  return { schema: 'link-stack-audit-v1', link_stack_bytes: LINK_STACK_BYTES, budget_bytes: budget,
    thread_entry_allowance_bytes: THREAD_ENTRY_ALLOWANCE_BYTES, deepest_normal_path_bytes: total,
    headroom_bytes: budget - total, deepest_symbols: deepest.symbols, unresolved_targets: deepest.unresolved_targets,
    complete_callgraph_bound: false, excluded_edges: 'panic_abort_logging',
    result: total <= budget ? 'selected_path_with_headroom' : 'blocked' };
}
