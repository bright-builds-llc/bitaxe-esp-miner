import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { completedFixture } from "./completed-fixture.mjs";
import { finalize, review } from "./finalize.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { judgeContinuity } from "./continuity-judge.mjs";
import { judgeAccounting, requireChannelNoWork } from "./accounting-judge.mjs";
import { readJournal } from "./journal.mjs";
import { failedShareFixture } from "./failed-share-fixture.mjs";

test("fresh continuity independently rejects copied probe, changed flash and ledger consumption", async t => {
  const f = await completedFixture(t), rows = await readJournal(f.root, f.context);
  const restoration = (await proof(f.root, "restoration.json")).value;
  const accounting = await judgeAccounting(f.root, f.context, rows, restoration);
  const inspect = () => judgeContinuity(f.root, f.context, rows, accounting.before, accounting.initial);
  assert.equal((await inspect()).installations, 5);
  await requireChannelNoWork(f.root, rows, accounting.before, accounting.after);
  const cases = [
    ["cycle-4.probe.json", value => { value.observation.observedAtUs = 1000; }, "v2_probe_evidence"],
    ["install-3/flash-command-evidence.json", value => { value.trusted_output = false; }, null],
    ["cycle-2.json", value => { value.report.settings_match = false; }, null],
  ];
  for (const [name, mutate, code] of cases) {
    const path = resolve(f.root, name), bytes = await readFile(path), value = JSON.parse(bytes);
    mutate(value); await writeFile(path, JSON.stringify(value));
    if (code) await assert.rejects(inspect(), { code }); else await assert.rejects(inspect());
    await writeFile(path, bytes);
  }
  const changed = structuredClone(accounting.after); changed.ledger.total_charged_ms++;
  await assert.rejects(requireChannelNoWork(f.root, rows, accounting.before, changed), { code: "v2_channel_ledger_changed" });
  assert.equal((await inspect()).cycles, 4);
});

test("complete Channel composes actual continuity and receipt readers, seals privately and reviews unchanged", async t => {
  const f = await completedFixture(t);
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  if (result.status !== "passed") assert.fail(JSON.stringify((await proof(f.root, "judgment-failure.json")).value));
  assert.equal(result.scope, "channel"); assert.equal(result.hardware_qualified, true);
  const projection = (await proof(f.root, "projection.json")).value;
  assert.equal(projection.counts.installations, 5); assert.equal(projection.counts.continuityCycles, 4);
  assert.equal(projection.counts.workDispatched, 0);
  assert.deepEqual(await review(f.root, f.operations), result);
  await assert.rejects(readFile(resolve(f.context.firmware_root, `docs/parity/evidence/str005-v2-serial/channel-${String(f.context.hostOrdinal).padStart(3, "0")}.json`)), { code: "ENOENT" });
  const file = resolve(f.root, "projection.json"), bytes = await readFile(file);
  projection.counts.workDispatched = 1; await writeFile(file, JSON.stringify(projection));
  await assert.rejects(review(f.root, f.operations), { code: "v2_projection_changed" });
  await writeFile(file, bytes); await review(f.root, f.operations);
});

test("failed Share publishes only accepted Channel after actual restoration and closed nonzero fixture exit", async t => {
  for (const cleanupComplete of [true, false]) {
    const channel = await completedFixture(t), channelResult = await finalize(channel.root, `${channel.root}.cleanup/receipt.json`, channel.operations);
    assert.equal(channelResult.status, "passed");
    const f = await failedShareFixture(t, channel, channelResult);
    assert.notEqual(f.baselineId, channel.baselineId);
    if (!cleanupComplete) {
      const path = `${f.root}.cleanup/receipt.json`, value = JSON.parse(await readFile(path));
      value.deviceBaselineConfirmed = false; await writeFile(path, JSON.stringify(value));
    }
    const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
    assert.equal(result.status, "unverified"); assert.equal(result.hardware_qualified, false);
    assert.deepEqual(result.publication.publishedScopes, cleanupComplete ? ["channel"] : []);
    assert.equal((await review(f.root, f.operations)).status, "unverified");
    assert.equal((await review(channel.root, channel.operations)).result_sha256, channelResult.result_sha256);
    const publicRoot = resolve(f.context.firmware_root, "docs/parity/evidence/str005-v2-serial");
    const channelName = `channel-${String(channel.context.hostOrdinal).padStart(3, "0")}.json`;
    if (cleanupComplete) assert.equal(JSON.parse(await readFile(resolve(publicRoot, channelName))).status, "accepted");
    else await assert.rejects(readFile(resolve(publicRoot, channelName)), { code: "ENOENT" });
    await assert.rejects(readFile(resolve(publicRoot, "share-001.json")), { code: "ENOENT" });
    await assert.rejects(readFile(resolve(f.root, "projection.json")), { code: "ENOENT" });
  }
});
