// Control owner stack audit: deepest resolved normal path plus routing-frame caps.
import { parseNoiseFrames } from './virtual-emulator/noise-stack-audit.mjs';

export const CONTROL_STACK_BYTES = 16 * 1024;
export const MARGIN_BYTES = 2048;
// pthread_task_func, thread_start, the call_once shim and the backtrace frame.
export const THREAD_ENTRY_ALLOWANCE_BYTES = 256;
const OWNER = 'bitaxe_firmware::bwg_worker_usb::run_owner';
// Routing frames every control command pays for; inlining a route target into
// them would silently re-inflate every path.
export const ROUTING_CAPS = Object.freeze([
  ['bitaxe_worker_control::controller::frame::<impl bitaxe_worker_control::controller::WorkerControl<V,S>>::prepare_frame', 1024],
  ['bitaxe_worker_control::controller::WorkerControl<V,S>::prepare_controller', 512],
  ['bitaxe_worker_control::controller::v2::<impl bitaxe_worker_control::controller::WorkerControl<V,S>>::prepare_v2', 256],
]);
// A panic aborts and reboots; logging is outside the control budget's normal paths.
const EXCLUDED = /panic|_fail$|unwind|backtrace|abort|expect_failed|unwrap_failed|handle_alloc_error|capacity_overflow|__assert|esp_log|printf/u;

/** Longest resolved direct-call path from `root`; unresolved edges are counted, never credited. */
export function deepestNormalPath(frames, root) {
  const starts = [...(frames.byName.get(root) ?? [])];
  if (starts.length !== 1) throw Error('control_stack_root_missing');
  const memo = new Map();
  let unresolved = 0;
  const walk = (address, active) => {
    if (active.has(address)) return { bytes: 0, path: [] };
    if (memo.has(address)) return memo.get(address);
    const frame = frames.byAddress.get(address);
    if (!frame || frame.bytes === null) { unresolved++; return { bytes: 0, path: [] }; }
    if (EXCLUDED.test(frame.name)) return { bytes: 0, path: [] };
    active.add(address);
    let best = { bytes: 0, path: [] };
    for (const target of new Set(frame.calls)) {
      const child = walk(target, active);
      if (child.bytes > best.bytes) best = child;
    }
    active.delete(address);
    const result = { bytes: frame.bytes + best.bytes, path: [address, ...best.path] };
    memo.set(address, result);
    return result;
  };
  const deepest = walk(starts[0], new Set());
  return { bytes: deepest.bytes, symbols: deepest.path.map((address) => frames.byAddress.get(address).name),
    unresolved_targets: unresolved };
}

/** Selected lower bound only: indirect calls and excluded edges are not bounded. */
export function auditControlStack(disassembly, ownerSource) {
  if (!ownerSource.includes('const OWNER_STACK_BYTES: usize = 16 * 1024;')) throw Error('control_stack_contract');
  const frames = parseNoiseFrames(disassembly);
  const routing = ROUTING_CAPS.map(([name, cap]) => {
    const variants = [...(frames.byName.get(name) ?? [])];
    if (variants.length !== 1) throw Error('control_stack_routing_symbol');
    const bytes = frames.byAddress.get(variants[0]).bytes;
    return { symbol: name, frame_bytes: bytes, cap_bytes: cap, fit: bytes !== null && bytes <= cap };
  });
  const deepest = deepestNormalPath(frames, OWNER);
  const total = deepest.bytes + THREAD_ENTRY_ALLOWANCE_BYTES, budget = CONTROL_STACK_BYTES - MARGIN_BYTES;
  return { schema: 'control-stack-audit-v1', control_stack_bytes: CONTROL_STACK_BYTES, budget_bytes: budget,
    thread_entry_allowance_bytes: THREAD_ENTRY_ALLOWANCE_BYTES, deepest_normal_path_bytes: total,
    headroom_bytes: budget - total, deepest_symbols: deepest.symbols, unresolved_targets: deepest.unresolved_targets,
    routing, complete_callgraph_bound: false, excluded_edges: 'panic_abort_logging',
    result: total <= budget && routing.every((item) => item.fit) ? 'selected_path_with_headroom' : 'blocked' };
}
