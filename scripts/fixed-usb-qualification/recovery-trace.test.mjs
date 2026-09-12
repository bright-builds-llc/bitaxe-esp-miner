import test from "node:test";
import assert from "node:assert/strict";
import { chmod, mkdtemp, readdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { digest, writeNew } from "./contract.mjs";
import { requireBeforeRecoveryTraces, requireRecoveryTraces, saveRecoveryTrace, validateRecoveryTrace } from "./recovery-trace.mjs";
import { RECOVERY_SCHEMA } from "./recovery-judge.mjs";

async function fixture(t) {
  const root = await mkdtemp(resolve(tmpdir(), "recovery-trace-"));
  await chmod(root, 0o700);
  t.after(() => rm(root, { recursive: true, force: true }));
  const context = { schema: RECOVERY_SCHEMA, recovery_phase: "loss", qualification_attempt: { ordinal: 14 } };
  const state = { connected: true, running: false, deviceLeaseInactive: true, deviceBaselineConfirmed: true, qualification: { attempt: { ordinal: 13 } } };
  const work = { ...state, running: true, qualification: { attempt: { ordinal: 14 }, work_dispatched: 1, safe_stop_complete: false } };
  const released = { ...work, connected: false, running: false, serialOwnershipReleased: true };
  const recovered = { ...state, qualification: { ...work.qualification, safe_stop_complete: true } };
  const records = [state, work, released, recovered].map((state, index) => ({ sequence: index + 1, state }));
  const fault = { after_sequence: 2, released_sequence: 3 };
  const trace = { snapshotAvailable: true, current: { epoch: 7 }, previous: { epoch: 6, events: [{ stage: "writer_abandoned" }] } };
  return { root, context, state, trace, records, fault };
}

test("trace shape rejects raw fields before starting its parser child", async () => {
  await assert.rejects(validateRecoveryTrace({ stage: "before", source: "browser", trace: {}, payload: "fixture" }, "/missing"),
    { code: "object_fields" });
});

test("invalid metadata is rejected before a file can persist it", async (t) => {
  const f = await fixture(t);
  await assert.rejects(saveRecoveryTrace(f.root, f.context, { stage: "before", source: "browser", trace: { payload: "fixture" } },
    1, f.state, async () => { throw new Error("parser_rejected"); }));
  assert.deepEqual(await readdir(f.root), []);
});

test("separate source exports stay private and cannot overwrite a captured trace", async (t) => {
  const f = await fixture(t);
  const input = { stage: "before", source: "device", trace: f.trace };
  await saveRecoveryTrace(f.root, f.context, input, 1, f.state, async () => f.trace);
  const path = resolve(f.root, "recovery-trace-before-device.json");
  const bytes = await readFile(path);
  assert.equal((await stat(path)).mode & 0o777, 0o600);
  await assert.rejects(saveRecoveryTrace(f.root, f.context, input, 2, f.state, async () => f.trace));
  assert.equal(digest(await readFile(path)), digest(bytes));
});

async function captured(t) {
  const f = await fixture(t);
  for (const stage of ["before", "loss", "recovered"]) {
    for (const source of stage === "loss" ? ["browser"] : ["browser", "device"]) {
      const sequence = { before: 1, loss: 3, recovered: 4 }[stage];
      const trace = source === "browser" ? { events: [{ epoch: stage === "recovered" ? 11 : 10,
        stage: stage === "loss" ? "close_completed" : "frame_delivered" }] } :
        stage === "recovered" ? { ...f.trace, current: { epoch: 8 }, previous: { epoch: 7, events: [{ stage: "writer_abandoned" }] } } : f.trace;
      await saveRecoveryTrace(f.root, f.context, { stage, source, trace }, sequence, f.records[sequence - 1].state, async () => trace);
    }
  }
  return f;
}

async function changeTrace(f, stage, source, change) {
  const path = resolve(f.root, `recovery-trace-${stage}-${source}.json`);
  const value = JSON.parse(await readFile(path, "utf8"));
  change(value);
  await writeFile(path, JSON.stringify(value));
}

test("trace completion requires retained previous-epoch evidence and binds every source hash", async (t) => {
  const f = await captured(t);
  const bindings = await requireRecoveryTraces(f.root, f.context, f.records, f.fault);
  assert.equal(bindings.length, 5);
  const path = resolve(f.root, "recovery-trace-recovered-device.json");
  const value = JSON.parse(await readFile(path, "utf8"));
  value.trace.previous = null;
  await writeFile(path, JSON.stringify(value));
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_prior_epoch_missing" });
});

test("missing traces have a typed judgment failure and a sealed attempt refuses later exports", async (t) => {
  const f = await fixture(t);
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_missing" });
  await writeNew(resolve(f.root, "result.json"), {});
  await assert.rejects(saveRecoveryTrace(f.root, f.context, { stage: "before", source: "browser", trace: {} },
    1, f.state, async () => f.trace));
  assert.deepEqual(await readdir(f.root), ["result.json"]);
});

test("retrospective baseline traces after the work checkpoint cannot establish before evidence", async t => {
  const f = await captured(t);
  f.records.push({ sequence: 5, state: f.state });
  await changeTrace(f, "before", "browser", value => { value.after_sequence = 5; });
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_order" });
});
test("all stages at the same initial row cannot replace the actual loss and recovery sequence", async t => {
  const f = await captured(t);
  await changeTrace(f, "loss", "browser", value => { value.after_sequence = 1; });
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_loss_binding" });
});
test("loss trace cannot precede the admitted release boundary", async t => {
  const f = await captured(t);
  await changeTrace(f, "loss", "browser", value => { value.after_sequence = 2; });
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_loss_binding" });
});
test("trace sequence must identify an actual journal row", async t => {
  const f = await captured(t);
  await changeTrace(f, "recovered", "browser", value => { value.after_sequence = 999; });
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_journal_binding" });
});
test("recovered device trace must retain the exact pre-loss device epoch", async t => {
  const f = await captured(t);
  await changeTrace(f, "recovered", "device", value => { value.trace.previous.epoch = 6; });
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_epoch_binding" });
});
test("browser close must belong to the pre-loss browser epoch", async t => {
  const f = await captured(t);
  await changeTrace(f, "loss", "browser", value => { value.trace.events[0].epoch = 9; });
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_browser_epoch" });
});
test("recovered browser trace must advance beyond the closed epoch", async t => {
  const f = await captured(t);
  await changeTrace(f, "recovered", "browser", value => { value.trace.events[0].epoch = 10; });
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_browser_epoch" });
});
test("recovered trace cannot precede confirmed device shutdown", async t => {
  const f = await captured(t);
  f.records[3].state.qualification.safe_stop_complete = false;
  await assert.rejects(requireRecoveryTraces(f.root, f.context, f.records, f.fault), { code: "recovery_trace_stop_missing" });
});
test("a before trace cannot be captured after issuance", async t => {
  const f = await fixture(t);
  await writeNew(resolve(f.root, "issued.json"), { ordinal: 14 });
  await assert.rejects(saveRecoveryTrace(f.root, f.context, { stage: "before", source: "device", trace: f.trace },
    1, f.state, async () => f.trace), { code: "private_path_exists" });
  assert.deepEqual(await readdir(f.root), ["issued.json"]);
});
test("completed current-attempt work cannot be relabeled as a pre-work trace", async t => {
  const f = await fixture(t);
  await assert.rejects(saveRecoveryTrace(f.root, f.context, { stage: "before", source: "device", trace: f.trace },
    4, f.records[3].state, async () => f.trace), { code: "recovery_trace_before_work" });
  assert.deepEqual(await readdir(f.root), []);
});
test("before signing requires both baseline trace sources bound to an inactive journal state", async t => {
  const f = await captured(t);
  await requireBeforeRecoveryTraces(f.root, f.context, f.records);
  f.records[0].state.deviceLeaseInactive = false;
  await assert.rejects(requireBeforeRecoveryTraces(f.root, f.context, f.records), { code: "recovery_before_trace_binding" });
});
test("an unavailable pre-work device trace stops signing", async t => {
  const f = await captured(t);
  await changeTrace(f, "before", "device", value => { value.trace.snapshotAvailable = false; });
  await assert.rejects(requireBeforeRecoveryTraces(f.root, f.context, f.records), { code: "recovery_trace_unavailable" });
});

test("same-epoch abandonment and revocation support link closure in either observed order", async t => {
  for (const stages of [["writer_abandoned", "epoch_revoked"], ["epoch_revoked", "writer_abandoned"]]) {
    const f = await captured(t);
    await changeTrace(f, "recovered", "device", value => {
      value.trace.previous.events = stages.map(stage => ({ stage, epoch: 7 }));
    });
    const bindings = await requireRecoveryTraces(f.root, f.context, f.records, f.fault);
    assert.equal(bindings.find(value => value.stage === "recovered" && value.source === "device").link_closed_verified, true);
  }
});
test("a missing stage or another epoch cannot establish trace-backed link closure", async t => {
  for (const events of [
    [{ stage: "writer_abandoned", epoch: 7 }],
    [{ stage: "writer_abandoned", epoch: 7 }, { stage: "epoch_revoked", epoch: 8 }],
  ]) {
    const f = await captured(t);
    await changeTrace(f, "recovered", "device", value => { value.trace.previous.events = events; });
    const bindings = await requireRecoveryTraces(f.root, f.context, f.records, f.fault);
    assert.equal(bindings.find(value => value.stage === "recovered" && value.source === "device").link_closed_verified, false);
  }
});
