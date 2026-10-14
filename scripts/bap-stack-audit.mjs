// BAP owner thread stack audit: deepest resolved normal path from the BAP loop.
import { parseNoiseFrames } from './virtual-emulator/noise-stack-audit.mjs';
import { deepestNormalPath, MARGIN_BYTES, THREAD_ENTRY_ALLOWANCE_BYTES } from './control-stack-audit.mjs';
import { withAddressTakenCallees } from './link-stack-audit.mjs';

const OWNER = 'bitaxe_firmware::bap_adapter::run_owner';
const STACK = /const BAP_OWNER_STACK_BYTES: usize = (\d+) \* 1_024;/u;

/**
 * Selected lower bound only: indirect calls and excluded edges are not bounded. The
 * stack size is read from the adapter source, so a resized thread is audited as built.
 */
export function auditBapStack(disassembly, adapterSource) {
  const maybeStack = STACK.exec(adapterSource);
  if (!maybeStack || !adapterSource.includes('.stack_size(BAP_OWNER_STACK_BYTES)')) throw Error('bap_stack_contract');
  const stackBytes = Number(maybeStack[1]) * 1024;
  const frames = withAddressTakenCallees(disassembly, parseNoiseFrames(disassembly));
  const deepest = deepestNormalPath(frames, OWNER);
  const total = deepest.bytes + THREAD_ENTRY_ALLOWANCE_BYTES, budget = stackBytes - MARGIN_BYTES;
  return { schema: 'bap-stack-audit-v1', bap_stack_bytes: stackBytes, budget_bytes: budget,
    thread_entry_allowance_bytes: THREAD_ENTRY_ALLOWANCE_BYTES, deepest_normal_path_bytes: total,
    headroom_bytes: budget - total, deepest_symbols: deepest.symbols, unresolved_targets: deepest.unresolved_targets,
    complete_callgraph_bound: false, excluded_edges: 'panic_abort_logging', address_taken_callees: true,
    result: total <= budget ? 'selected_path_with_headroom' : 'blocked' };
}
