// Serial writer thread stack audit: deepest resolved normal path from the writer loop.
import { parseNoiseFrames } from './virtual-emulator/noise-stack-audit.mjs';
import { deepestNormalPath, MARGIN_BYTES, THREAD_ENTRY_ALLOWANCE_BYTES } from './control-stack-audit.mjs';
import { withAddressTakenCallees } from './link-stack-audit.mjs';

export const WRITER_STACK_BYTES = 8192;
const WRITER = 'bitaxe_firmware::bwg_worker_usb::writer::run';
// The writer loop samples credit every turn; no other thread owns a record writer.
const WRITER_ANCHOR = 'bitaxe_worker_control::serial::writer::SerialRecordWriter::receive_credit';
const CONTRACT = /\.name\("bwg-serial-writer"\.into\(\)\)\s*\.stack_size\(8192\)/u;

/** Selected lower bound only: indirect calls and excluded edges are not bounded. */
export function auditWriterStack(disassembly, ownerSource) {
  if (!CONTRACT.test(ownerSource)) throw Error('writer_stack_contract');
  const frames = withAddressTakenCallees(disassembly, parseNoiseFrames(disassembly));
  const deepest = deepestNormalPath(frames, WRITER, WRITER_ANCHOR);
  const total = deepest.bytes + THREAD_ENTRY_ALLOWANCE_BYTES, budget = WRITER_STACK_BYTES - MARGIN_BYTES;
  return { schema: 'writer-stack-audit-v1', writer_stack_bytes: WRITER_STACK_BYTES, budget_bytes: budget,
    thread_entry_allowance_bytes: THREAD_ENTRY_ALLOWANCE_BYTES, deepest_normal_path_bytes: total,
    headroom_bytes: budget - total, root_frame: deepest.root_frame, deepest_symbols: deepest.symbols,
    unresolved_targets: deepest.unresolved_targets, complete_callgraph_bound: false,
    excluded_edges: 'panic_abort_logging', address_taken_callees: true,
    result: total <= budget ? 'selected_path_with_headroom' : 'blocked' };
}
