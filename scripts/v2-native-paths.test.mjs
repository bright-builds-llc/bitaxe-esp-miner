import assert from "node:assert/strict";
import test from "node:test";
import { auditV2Paths, V2_CHANNEL_ENTRY, V2_SHARE_ENTRY } from "./v2-native-paths.mjs";
import { parseNativeFunctions } from "./telemetry-stack-audit.mjs";
const worker = "bitaxe_firmware::production_mining_session::transport::run_worker";
const borrow = "bitaxe_firmware::production_mining_session::transport::borrow::NoiseBorrowWorker::run_job";
const sources = {
  transportSource: "const WORKER_STACK_BYTES: usize = 12 * 1024; .stack_size(WORKER_STACK_BYTES) borrow.run_job()",
  borrowSource: "(callbacks.run)(); (callbacks.complete)();",
  channelSource: "owner.external = true",
  runtimeSource: '#[export_name = "bitaxe_v2_channel_owner_entry"] fn channel_entry()',
  noiseOwnerSource: ".dispatch(if owner.external { crate::v2_serial_runtime::channel_entry } else { owner_entry }, job_completed,)",
};
function fixture({ verify = 512, prepare = true } = {}) {
  const rows = [[worker, 32, [1, 3]], [borrow, 32, []], [V2_CHANNEL_ENTRY, 32, prepare ? [4, 5] : [5]],
    [V2_SHARE_ENTRY, 32, prepare ? [4, 5] : [5]], ["rustsecp256k1_v0_9_2_ellswift_encode", 32, []],
    ["noise_sv2::initiator::Initiator::step_2_with_now", 32, [6]], ["rustsecp256k1_v0_9_2_schnorrsig_verify", verify, []],
    ["std::sys::backtrace::__rust_begin_short_backtrace", 32, [0]],
    ["core::ops::function::FnOnce::call_once{{vtable.shim}}", 32, [7]],
    ["std::sys::pal::unix::thread::Thread::new::thread_start", 32, []], ["pthread_task_func", 32, []]];
  const address = index => 0x42001000 + index * 0x100;
  const hex = value => value.toString(16);
  const text = rows.map(([name, bytes, targets], index) => `${hex(address(index))} <${name}>:\n${hex(address(index))}: 000000 entry a1, ${bytes}\n`
    + targets.map((target, offset) => `${hex(address(index) + 3 * (offset + 1))}: 000000 call8 ${hex(address(target))} <${rows[target][0]}>\n`).join("")
    + `${hex(address(index) + 3 * (targets.length + 1))}: 000000 retw.n\n`).join("\n");
  return parseNativeFunctions(text);
}

test("both scopes retain measured caller costs and explicit partial-callgraph limits", () => {
  // Arrange / Act
  const result = auditV2Paths(fixture(), sources);
  // Assert
  assert.deepEqual(Object.keys(result), ["channel", "share"]);
  assert.equal(result.channel.selectedPathBytes - result.share.selectedPathBytes, 32);
  assert.equal(result.share.addedStackBytes, 0);
  assert.equal(result.share.completeCallgraphBound, false);
});

test("the exact512-byte platform margin passes and one extra frame quantum fails", () => {
  assert.equal(auditV2Paths(fixture({ verify: 11520 }), sources).channel.remainingStackBytes, 512);
  assert.throws(() => auditV2Paths(fixture({ verify: 11536 }), sources), { code: "v2_native_selected_path_budget" });
});

test("missing crypto or caller edges remain unproved instead of zero-cost", () => {
  assert.throws(() => auditV2Paths(fixture({ prepare: false }), sources), { code: "v2_native_crypto_unproven" });
  const functions = fixture(); functions.get(0x42001000).instructions = functions.get(0x42001000).instructions.filter(row => !row.args.includes(borrow));
  assert.throws(() => auditV2Paths(functions, sources), { code: "v2_native_borrow_edge" });
});

test("new owners, changed stack budgets and callback registration drift reject qualification", () => {
  for (const [change, code] of [
    [{ channelSource: sources.channelSource + " thread::Builder" }, "v2_native_new_owner"],
    [{ transportSource: sources.transportSource.replace("12 * 1024", "24 * 1024") }, "v2_native_stack_declaration"],
    [{ noiseOwnerSource: sources.noiseOwnerSource.replace("channel_entry", "unknown") }, "v2_native_channel_registration"],
  ]) assert.throws(() => auditV2Paths(fixture(), { ...sources, ...change }), { code });
});
