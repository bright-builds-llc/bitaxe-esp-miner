import { parseNativeFunctions, nativeFrame } from './telemetry-stack-audit.mjs';
import { noiseNativeCalls } from './noise-native-calls.mjs';

const check = (ok, code) => { if (!ok) throw Error(code); };
export const CONTROL_STACK_BYTES = 16384;
export const MINIMUM_HEADROOM_BYTES = 2048;
const OWNER = 'bitaxe_firmware::bwg_worker_usb::run_owner';
const FRAME = /controller::frame::.*::prepare_frame$/u;
const START = /controller::start_dispatch::.*::prepare_start_controller$/u;
const GENERAL = /WorkerControl<V,S>::prepare_controller$/u;
const VERIFY = /WorkLeaseAuthorizationVerifier<.*>::verify_start$/u;
const TAIL = [
  /^ed25519_dalek::verifying::VerifyingKey::verify_strict$/u,
  /^ed25519_dalek::verifying::RCompute<CtxDigest>::compute$/u,
  /^curve25519_dalek::edwards::EdwardsPoint::vartime_double_scalar_mul_basepoint$/u,
  /^curve25519_dalek::backend::vartime_double_base_mul$/u,
  /^<curve25519_dalek::window::NafLookupTable5<.*ProjectiveNielsPoint>.*>::from$/u,
  /^curve25519_dalek::backend::serial::curve_models::ProjectivePoint::double$/u,
  /^curve25519_dalek::backend::serial::u32::field::FieldElement2625::square_inner$/u,
];

/** Selected signature-verification ancestry only, never a global task-stack maximum. */
export function auditSignedStart(disassembly) {
  const functions = parseNativeFunctions(disassembly), values = [...functions.values()], cache = new Map();
  const calls = fn => {
    if (!cache.has(fn.address)) cache.set(fn.address, noiseNativeCalls(fn));
    return cache.get(fn.address);
  };
  const one = pattern => {
    const rows = values.filter(fn => typeof pattern === 'string' ? fn.symbol === pattern : pattern.test(fn.symbol));
    check(rows.length === 1, 'signed_start_symbol_missing_or_ambiguous'); return rows[0];
  };
  const edge = (caller, callee) => {
    const found = calls(caller).filter(row => row.target === callee.address);
    check(found.length > 0, 'signed_start_edge_missing');
    return { caller: caller.address, callee: callee.address, call_addresses: found.map(row => row.call_address), evidence: 'compiled_call' };
  };
  const parent = (child, pattern) => {
    const found = values.filter(fn => pattern.test(fn.symbol) && calls(fn).some(row => row.target === child.address));
    check(found.length === 1, 'signed_start_caller_missing_or_ambiguous'); return found[0];
  };
  const owner = one(OWNER), backtrace = parent(owner, /^std::sys::backtrace::__rust_begin_short_backtrace$/u);
  const shim = parent(backtrace, /^core::ops::function::FnOnce::call_once\{\{vtable.shim\}\}$/u);
  const pthread = one('pthread_task_func'), thread = one('std::sys::pal::unix::thread::Thread::new::thread_start');
  const frame = one(FRAME), verify = one(VERIFY);
  const starts = values.filter(fn => START.test(fn.symbol)); check(starts.length <= 1, 'signed_start_symbol_missing_or_ambiguous');
  const outlined = starts.length === 1, dispatcher = outlined ? starts[0] : one(GENERAL);
  const nodes = [pthread, thread, shim, backtrace, owner, frame, dispatcher];
  const edges = [
    { caller: pthread.address, callee: thread.address, evidence: 'source_abi_supported_dynamic_function_pointer' },
    { caller: thread.address, callee: shim.address, evidence: 'source_abi_supported_dynamic_vtable' },
  ];
  for (let i = 2; i + 1 < nodes.length; i++) edges.push(edge(nodes[i], nodes[i + 1]));
  // LLVM may leave start out of line. Admit only that explicit bridge, not arbitrary callees.
  if (!calls(dispatcher).some(row => row.target === verify.address)) {
    const bridge = one(/WorkerControl<V,S>::start$/u);
    edges.push(edge(dispatcher, bridge)); nodes.push(bridge);
  }
  edges.push(edge(nodes.at(-1), verify)); nodes.push(verify);
  for (const pattern of TAIL) { const next = one(pattern); edges.push(edge(nodes.at(-1), next)); nodes.push(next); }
  const frames = nodes.map(nativeFrame), bytes = frames.reduce((a, b) => a + b, 0);
  const headroom = CONTROL_STACK_BYTES - bytes;
  const isolated = outlined && nodes.every(fn => !GENERAL.test(fn.symbol));
  return { schema: 'signed-start-stack-audit-v1', result: headroom < MINIMUM_HEADROOM_BYTES ? 'budget_exceeded' :
    !isolated ? 'start_dispatch_not_isolated' : 'selected_path_with_headroom', control_stack_bytes: CONTROL_STACK_BYTES,
    selected_fixed_entry_bytes: bytes, unclaimed_headroom_bytes: headroom, required_headroom_bytes: MINIMUM_HEADROOM_BYTES,
    start_dispatch_isolated: isolated, platform_edges_native_resolved: false, complete_callgraph_bound: false,
    observed_fault_trace: false, hardware_verified: false,
    nodes: nodes.map((fn, index) => ({ symbol: fn.symbol, address: fn.address.toString(16), entry_bytes: frames[index] })),
    edges: edges.map(row => ({ ...row, caller: row.caller.toString(16), callee: row.callee.toString(16),
      ...(row.call_addresses ? { call_addresses: row.call_addresses.map(at => at.toString(16)) } : {}) })) };
}
