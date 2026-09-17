import { nativeFrame } from "./telemetry-stack-audit.mjs";
import { noiseNativeCalls, noiseNativeInstructions } from "./noise-native-calls.mjs";
import { selectedNoiseSymbol } from "./noise-native-disassembly.mjs";

export const V2_CHANNEL_ENTRY = "bitaxe_v2_channel_owner_entry";
export const V2_SHARE_ENTRY = "bitaxe_firmware::production_mining_session::transport::v2::run";
export const V2_ROOTS = Object.freeze([V2_CHANNEL_ENTRY, V2_SHARE_ENTRY]);
export const selectedV2Symbol = (symbol) => selectedNoiseSymbol(symbol) || V2_ROOTS.includes(symbol) ||
  symbol.startsWith("bitaxe_firmware::v2_serial_runtime::") ||
  symbol.startsWith("bitaxe_firmware::noise_serial_runtime::") ||
  symbol.startsWith("bitaxe_stratum::v2::standard_io::");
const check = (condition, code) => { if (!condition) throw Object.assign(new Error(code), { code }); };
const calls = (fn) => noiseNativeCalls(fn, { compilerPrivateSpills: true });
const frame = (fn) => nativeFrame({ ...fn, instructions: noiseNativeInstructions(fn) });

/** Explicit native callback/caller paths only; omitted allocators and indirect paths are not certified. */
export function auditV2Paths(functions, { transportSource, borrowSource, channelSource, runtimeSource, noiseOwnerSource }, scopes = ["channel", "share"]) {
  check(scopes.length > 0 && new Set(scopes).size === scopes.length && scopes.every((scope) => ["channel", "share"].includes(scope)), "v2_native_scope");
  check(/const WORKER_STACK_BYTES:\s*usize\s*=\s*12\s*\*\s*1024\s*;/u.test(transportSource) &&
    /\.stack_size\(WORKER_STACK_BYTES\)/u.test(transportSource), "v2_native_stack_declaration");
  check(!/thread::Builder|\.spawn\(/u.test(channelSource + runtimeSource), "v2_native_new_owner");
  check(transportSource.includes("borrow.run_job()") && borrowSource.includes("(callbacks.run)();") &&
    borrowSource.includes("(callbacks.complete)();"), "v2_native_callback_binding");
  check(/\.dispatch\(\s*if owner\.external \{\s*crate::v2_serial_runtime::channel_entry\s*\} else \{\s*owner_entry\s*\},\s*job_completed,\s*\)/u.test(noiseOwnerSource) &&
    channelSource.includes("owner.external = true") && /#\[export_name = "bitaxe_v2_channel_owner_entry"\][\s\S]*?fn channel_entry\(\)/u.test(runtimeSource),
  "v2_native_channel_registration");
  const all = [...functions.values()];
  const one = (symbol) => {
    const matches = all.filter((fn) => fn.symbol === symbol);
    check(matches.length === 1, "v2_native_path_symbol"); return matches[0];
  };
  const worker = one("bitaxe_firmware::production_mining_session::transport::run_worker");
  const borrow = one("bitaxe_firmware::production_mining_session::transport::borrow::NoiseBorrowWorker::run_job");
  const caller = (callee, symbol) => {
    const matches = all.filter((fn) => fn.symbol === symbol && calls(fn).some((edge) => edge.target === callee.address));
    check(matches.length === 1, "v2_native_thread_caller"); return matches[0];
  };
  const backtrace = caller(worker, "std::sys::backtrace::__rust_begin_short_backtrace");
  const shim = caller(backtrace, "core::ops::function::FnOnce::call_once{{vtable.shim}}");
  const common = [one("pthread_task_func"), one("std::sys::pal::unix::thread::Thread::new::thread_start"), shim, backtrace, worker];
  check(calls(worker).some((edge) => edge.target === borrow.address), "v2_native_borrow_edge");
  check(calls(worker).some((edge) => edge.target === one(V2_SHARE_ENTRY).address), "v2_native_share_edge");
  const results = {};
  for (const [scope, symbol] of [["channel", V2_CHANNEL_ENTRY], ["share", V2_SHARE_ENTRY]]) {
    if (!scopes.includes(scope)) continue;
    const memo = new Map(), active = new Set(), observed = new Set();
    const total = (path) => path.reduce((sum, node) => sum + node.entryBytes, 0);
    function visit(fn) {
      check(!active.has(fn.address), "v2_native_selected_recursion");
      if (memo.has(fn.address)) return memo.get(fn.address);
      check(observed.size < 4096, "v2_native_selected_bound");
      active.add(fn.address); observed.add(fn.symbol);
      let suffix = [];
      for (const edge of calls(fn)) {
        const target = functions.get(edge.target);
        if (target && selectedV2Symbol(target.symbol)) {
          const candidate = visit(target); if (total(candidate) > total(suffix)) suffix = candidate;
        }
      }
      const result = [{ symbol: fn.symbol, entryBytes: frame(fn) }, ...suffix];
      active.delete(fn.address); memo.set(fn.address, result); return result;
    }
    const path = visit(one(symbol));
    check([...observed].some((name) => name.includes("schnorrsig_verify")) &&
      [...observed].some((name) => name.includes("ellswift_encode")), "v2_native_crypto_unproven");
    const prefix = scope === "channel" ? [...common, borrow] : common;
    const nodes = prefix.map((fn) => ({ symbol: fn.symbol, entryBytes: frame(fn) })).concat(path);
    const selectedPathBytes = total(nodes);
    if (selectedPathBytes + 512 > 12288) throw Object.assign(new Error("v2_native_selected_path_budget"), {
      code: "v2_native_selected_path_budget", scope, selectedPathBytes, requiredMarginBytes: 512, nodes,
    });
    results[scope] = { symbol, ownership: "existing_primary_transport_worker", stackBytes: 12288, addedStackBytes: 0,
      selectedPathBytes, requiredMarginBytes: 512, remainingStackBytes: 12288 - selectedPathBytes,
      selectedFunctions: memo.size, nodes, completeCallgraphBound: false };
  }
  return results;
}
