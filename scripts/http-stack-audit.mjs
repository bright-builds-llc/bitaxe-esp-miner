// HTTP server task stack audit: ESP-IDF dispatch frames plus the deepest resolved
// normal path of every registered request handler.
import { parseNoiseFrames } from './virtual-emulator/noise-stack-audit.mjs';
import { deepestNormalPath, MARGIN_BYTES, THREAD_ENTRY_ALLOWANCE_BYTES } from './control-stack-audit.mjs';
import { withAddressTakenCallees } from './link-stack-audit.mjs';

/** Every registered handler monomorphizes this; all instances share one demangled name. */
export const HANDLER = '<embedded_svc::http::server::FnHandler<F> as embedded_svc::http::server::Handler<C>>::handle';
// httpd's task loop down to the Rust trampoline that calls a handler through a pointer.
export const DISPATCH = Object.freeze(['httpd_thread', 'httpd_sess_process', 'httpd_uri',
  'esp_idf_svc::http::server::EspHttpServer::handle_req', 'esp_idf_svc::http::server::EspHttpServer::fn_handler']);
// Jobs queued with `httpd_queue_work` run from the server loop, whose control-message
// path is inlined into `httpd_thread`.
export const QUEUED_WORK = Object.freeze(['bitaxe_firmware::http_api::accessory_settings::run_accessory_job']);
export const QUEUED_DISPATCH = Object.freeze(['httpd_thread']);
const STACK = /const HTTP_SERVER_TASK_STACK_BYTES: usize = (\d+) \* 1024;/u;

/**
 * Selected lower bound only: indirect calls and excluded edges are not bounded. The
 * dispatch chain is charged at its largest variant of each frame.
 */
export function auditHttpStack(disassembly, serverSource) {
  const maybeStack = STACK.exec(serverSource);
  if (!maybeStack || !serverSource.includes('stack_size: HTTP_SERVER_TASK_STACK_BYTES')) throw Error('http_stack_contract');
  const stackBytes = Number(maybeStack[1]) * 1024;
  const frames = withAddressTakenCallees(disassembly, parseNoiseFrames(disassembly));
  const chargeChain = (names) => names.map((name) => {
    const sizes = [...(frames.byName.get(name) ?? [])].map((address) => frames.byAddress.get(address).bytes);
    if (!sizes.length || sizes.some((bytes) => bytes === null)) throw Error('http_dispatch_frame_missing');
    return Math.max(...sizes);
  }).reduce((sum, bytes) => sum + bytes, 0);
  const dispatch = chargeChain(DISPATCH);
  const handlers = [...(frames.byName.get(HANDLER) ?? [])];
  if (!handlers.length) throw Error('http_handlers_missing');
  let deepest = null;
  for (const address of handlers) {
    const alias = `${HANDLER}@${address.toString(16)}`;
    frames.byName.set(alias, new Set([address]));
    const path = deepestNormalPath(frames, alias);
    if (!deepest || dispatch + path.bytes > deepest.total) deepest = { ...path, total: dispatch + path.bytes };
  }
  const queuedDispatch = chargeChain(QUEUED_DISPATCH);
  for (const root of QUEUED_WORK) {
    if (!frames.byName.has(root)) throw Error('http_queued_work_missing');
    const path = deepestNormalPath(frames, root);
    if (queuedDispatch + path.bytes > deepest.total) deepest = { ...path, total: queuedDispatch + path.bytes };
  }
  const total = deepest.total + THREAD_ENTRY_ALLOWANCE_BYTES, budget = stackBytes - MARGIN_BYTES;
  return { schema: 'http-stack-audit-v1', http_stack_bytes: stackBytes, budget_bytes: budget,
    thread_entry_allowance_bytes: THREAD_ENTRY_ALLOWANCE_BYTES, dispatch_bytes: dispatch, handlers: handlers.length,
    queued_dispatch_bytes: queuedDispatch, queued_work: QUEUED_WORK.length,
    deepest_normal_path_bytes: total, headroom_bytes: budget - total, deepest_symbols: deepest.symbols,
    unresolved_targets: deepest.unresolved_targets, complete_callgraph_bound: false,
    excluded_edges: 'panic_abort_logging', address_taken_callees: true,
    result: total <= budget ? 'selected_path_with_headroom' : 'blocked' };
}
