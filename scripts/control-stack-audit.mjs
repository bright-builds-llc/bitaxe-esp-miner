// Control owner stack audit: deepest resolved normal path plus routing-frame caps.
import { parseNoiseFrames } from './virtual-emulator/noise-stack-audit.mjs';

export const CONTROL_STACK_BYTES = 16 * 1024;
export const MARGIN_BYTES = 2048;
// pthread_task_func, thread_start, the call_once shim and the backtrace frame. An owner inlined into
// that frame also counts it in its path, which only overstates depth.
export const THREAD_ENTRY_ALLOWANCE_BYTES = 256;
const OWNER = 'bitaxe_firmware::bwg_worker_usb::run_owner';
// Routing frames every control command pays for; inlining a route target into
// them would silently re-inflate every path.
export const ROUTING_CAPS = Object.freeze([
  ['bitaxe_worker_control::controller::frame::<impl bitaxe_worker_control::controller::WorkerControl<V,S>>::prepare_frame', 1024],
  ['bitaxe_worker_control::controller::WorkerControl<V,S>::prepare_controller', 512],
  ['bitaxe_worker_control::controller::v2::<impl bitaxe_worker_control::controller::WorkerControl<V,S>>::prepare_v2', 256],
]);
// The owner's routing entry; only the control thread calls it.
const OWNER_ANCHOR = ROUTING_CAPS[0][0];
// A panic aborts and reboots; logging is outside the control budget's normal paths.
const EXCLUDED = /panic|_fail$|unwind|backtrace|abort|expect_failed|unwrap_failed|handle_alloc_error|capacity_overflow|__assert|esp_log|printf/u;
// Every spawned closure monomorphizes its own instance; demangling gives them all this one name.
export const THREAD_TRAMPOLINE = 'std::sys::backtrace::__rust_begin_short_backtrace';

/** True when a direct-call path leads from `start` to `targets` without entering another thread's trampoline. */
function reachesWithinThread(frames, start, targets, trampolines) {
  const seen = new Set([start]), queue = [start];
  for (let index = 0; index < queue.length; index++) {
    if (targets.has(queue[index])) return true;
    for (const next of frames.byAddress.get(queue[index])?.calls ?? []) {
      if (seen.has(next) || trampolines.has(next)) continue;
      seen.add(next); queue.push(next);
    }
  }
  return false;
}

/**
 * The owner's frame. When LLVM inlines the owner into its spawn closure, the owner symbol disappears
 * and its body and frame live in one trampoline instance; that instance is the one that reaches
 * `maybeAnchor`, a callee only the owner's thread makes. No or several such instances fail closed.
 */
export function threadRoot(frames, root, maybeAnchor = null) {
  const named = [...(frames.byName.get(root) ?? [])];
  if (named.length === 1) return { address: named[0], inlined: false };
  if (named.length > 1 || maybeAnchor === null) throw Error('control_stack_root_missing');
  const anchors = new Set(frames.byName.get(maybeAnchor) ?? []);
  const trampolines = new Set(frames.byName.get(THREAD_TRAMPOLINE) ?? []);
  const reaching = [...trampolines].filter((start) => reachesWithinThread(frames, start, anchors, trampolines));
  if (reaching.length > 1) throw Error('control_stack_root_ambiguous');
  if (reaching.length === 0) throw Error('control_stack_root_missing');
  return { address: reaching[0], inlined: true };
}

/**
 * Longest resolved direct-call path from `root`; unresolved edges are counted, never credited.
 * With `maybeAnchor`, an owner inlined into its thread trampoline is still found (see `threadRoot`).
 */
export function deepestNormalPath(frames, root, maybeAnchor = null) {
  const start = threadRoot(frames, root, maybeAnchor);
  const memo = new Map();
  let unresolved = 0;
  const walk = (address, active) => {
    if (active.has(address)) return { bytes: 0, path: [] };
    if (memo.has(address)) return memo.get(address);
    const frame = frames.byAddress.get(address);
    if (!frame || frame.bytes === null) { unresolved++; return { bytes: 0, path: [] }; }
    // The root is the owner even when it is a trampoline, whose name `backtrace` would exclude.
    if (address !== start.address && EXCLUDED.test(frame.name)) return { bytes: 0, path: [] };
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
  const deepest = walk(start.address, new Set());
  return { bytes: deepest.bytes, symbols: deepest.path.map((address) => frames.byAddress.get(address).name),
    unresolved_targets: unresolved, root_frame: start.inlined ? 'inlined_into_thread_trampoline' : 'owner_symbol' };
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
  const deepest = deepestNormalPath(frames, OWNER, OWNER_ANCHOR);
  const total = deepest.bytes + THREAD_ENTRY_ALLOWANCE_BYTES, budget = CONTROL_STACK_BYTES - MARGIN_BYTES;
  return { schema: 'control-stack-audit-v1', control_stack_bytes: CONTROL_STACK_BYTES, budget_bytes: budget,
    thread_entry_allowance_bytes: THREAD_ENTRY_ALLOWANCE_BYTES, deepest_normal_path_bytes: total,
    headroom_bytes: budget - total, root_frame: deepest.root_frame, deepest_symbols: deepest.symbols,
    unresolved_targets: deepest.unresolved_targets, routing, complete_callgraph_bound: false,
    excluded_edges: 'panic_abort_logging',
    result: total <= budget && routing.every((item) => item.fit) ? 'selected_path_with_headroom' : 'blocked' };
}
